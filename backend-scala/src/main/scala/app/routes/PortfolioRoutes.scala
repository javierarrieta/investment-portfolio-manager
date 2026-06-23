package app.routes

import cats.effect.IO
import cats.effect.Resource
import cats.implicits._
import org.http4s._
import org.http4s.dsl.io._
import org.http4s.circe._
import org.http4s.server._
import io.circe._
import io.circe.generic.semiauto._
import io.circe.syntax._
import app.models._
import app.models.AssetOut._
import app.services.TaxLotEngine
import app.services.TaxLotEncoder._
import app.services.StatsEngine
import app.db.Repository
import doobie.hikari.HikariTransactor
import app.services.StatsResult

object PortfolioRoutes {

  implicit val portfolioEncoder: Encoder[Portfolio] = deriveEncoder[Portfolio]
  implicit val assetEncoder: Encoder[Asset] = deriveEncoder[Asset]
  implicit val transactionEncoder: Encoder[Transaction] = deriveEncoder[Transaction]
  implicit val historicalPriceEncoder: Encoder[HistoricalPrice] = deriveEncoder[HistoricalPrice]
  implicit val taxLotEncoder: Encoder[TaxLot] = deriveEncoder[TaxLot]
  implicit val portfolioSummaryEncoder: Encoder[PortfolioSummary] = deriveEncoder[PortfolioSummary]
  implicit val portfolioOutEncoder: Encoder[PortfolioOut] = deriveEncoder[PortfolioOut]

  implicit val createPortfolioRequestDecoder: Decoder[CreatePortfolioRequest] = deriveDecoder[CreatePortfolioRequest]
  implicit val createAssetRequestDecoder: Decoder[CreateAssetRequest] = deriveDecoder[CreateAssetRequest]
  implicit val createTransactionRequestDecoder: Decoder[CreateTransactionRequest] = deriveDecoder[CreateTransactionRequest]

  implicit val createPortfolioRequestDecoderCirce: EntityDecoder[IO, CreatePortfolioRequest] =
    jsonOf[IO, CreatePortfolioRequest]

  implicit val createAssetRequestDecoderCirce: EntityDecoder[IO, CreateAssetRequest] =
    jsonOf[IO, CreateAssetRequest]

  implicit val createTransactionRequestDecoderCirce: EntityDecoder[IO, CreateTransactionRequest] =
    jsonOf[IO, CreateTransactionRequest]

  def routes(
    repo: Repository[IO],
    xa: HikariTransactor[IO]
  ): HttpRoutes[IO] = {

    val taxLotEngine = TaxLotEngine.create(xa)
    val statsEngine = StatsEngine.create(xa)

    HttpRoutes.of[IO] {
      // Root endpoint
      case GET -> Root =>
        val json = Json.obj(
          "message" -> Json.fromString("Investment Portfolio API"),
          "version" -> Json.fromString("1.0.0")
        )
        Ok(json)

      // Portfolios
      case GET -> Root / "api" / "portfolios" =>
        repo.listPortfolios().flatMap { portfolios =>
          Ok(portfolios.map(p => PortfolioOut(
            p.name,
            p.description,
            p.currency,
            p.id.getOrElse(0),
            List.empty
          )).asJson)
        }

      case req @ POST -> Root / "api" / "portfolios" =>
        req.decode[CreatePortfolioRequest] { req =>
          repo.createPortfolio(req.toPortfolio).flatMap { portfolio =>
            Created(PortfolioOut(
              portfolio.name,
              portfolio.description,
              portfolio.currency,
              portfolio.id.getOrElse(0),
              List.empty
            ).asJson)
          }
        }

      case GET -> Root / "api" / "portfolios" / portfolioId =>
        repo.getPortfolio(portfolioId.toInt).flatMap {
          case Some(p) =>
            repo.listAssets(p.id.getOrElse(0)).flatMap { assets =>
              repo.listTransactions(p.id.getOrElse(0)).flatMap { transactions =>
                Ok(PortfolioOut(
                  p.name,
                  p.description,
                  p.currency,
                  p.id.getOrElse(0),
                  assets.map(a => AssetOut(
                    a.symbol,
                    a.name,
                    a.assetType,
                    a.sector,
                    a.id.getOrElse(0),
                    a.portfolioId,
                    List.empty
                  ))
                ).asJson)
              }
            }
          case None => NotFound(s"Portfolio not found: $portfolioId")
        }

      case DELETE -> Root / "api" / "portfolios" / portfolioId =>
        repo.deletePortfolio(portfolioId.toInt).flatMap {
          case true => NoContent()
          case false => NotFound(s"Portfolio not found: $portfolioId")
        }

      // Assets
      case req @ POST -> Root / "api" / "portfolios" / portfolioId / "assets" =>
        req.decode[CreateAssetRequest] { aReq =>
          repo.createAsset(aReq.toAsset(portfolioId.toInt)).flatMap { asset =>
            Created(asset.asJson)
          }
        }

      case DELETE -> Root / "api" / "assets" / assetId =>
        repo.deleteAsset(assetId.toInt).flatMap {
          case true => NoContent()
          case false => NotFound(s"Asset not found: $assetId")
        }

      // Transactions
      case req @ POST -> Root / "api" / "portfolios" / portfolioId / "assets" / assetId / "transactions" =>
        req.decode[CreateTransactionRequest] { tReq =>
          repo.createTransaction(tReq.toTransaction(assetId.toInt)).flatMap { transaction =>
            Created(transaction.asJson)
          }
        }

      case GET -> Root / "api" / "portfolios" / portfolioId / "transactions" =>
        repo.listTransactions(portfolioId.toInt).flatMap { transactions =>
          Ok(transactions.map(t => TransactionOut(
            t.`type`,
            t.quantity,
            t.price,
            t.fee,
            t.`date`,
            t.id.getOrElse(0),
            t.assetId
          )).asJson)
        }

      case DELETE -> Root / "api" / "transactions" / transactionId =>
        repo.deleteTransaction(transactionId.toInt).flatMap {
          case true => NoContent()
          case false => NotFound(s"Transaction not found: $transactionId")
        }

      // Tax Summary
      case req @ GET -> Root / "api" / "portfolios" / portfolioId / "tax-summary" =>
        val strategyParam = req.params.get("strategy")
        val thresholdParam = req.params.get("threshold_days")
        val strategy = strategyParam.getOrElse("FIFO")
        val thresholdDays = thresholdParam.map(_.toInt).getOrElse(30)
        taxLotEngine.getTaxSummary(portfolioId.toInt, strategy, thresholdDays).flatMap { summary =>
          Ok(summary.asJson)
        }

      // Performance
      case req @ GET -> Root / "api" / "portfolios" / portfolioId / "performance" =>
        val startDateParam = req.params.get("start_date")
        val endDateParam = req.params.get("end_date")
        val startDate = startDateParam.getOrElse(java.time.LocalDate.now().minusYears(1).toString)
        val endDate = endDateParam.getOrElse(java.time.LocalDate.now().toString)
        statsEngine.calculateStats(startDate, endDate).flatMap { stats =>
          Ok(StatsResult(
            startDate,
            endDate,
            stats.timeWeightedReturn,
            stats.annualizedReturn,
            stats.volatility,
            stats.sharpeRatio,
            stats.benchmarkReturn,
            stats.beta,
            stats.alpha
          ).asJson)
        }

      case _ =>
        NotFound("Endpoint not found")
    }
  }
}

case class CreatePortfolioRequest(name: String, description: Option[String] = None, currency: Option[String] = None) {
  def toPortfolio: Portfolio = Portfolio(None, name, description, currency.getOrElse("USD"), currency.getOrElse("USD"))
}

case class CreateAssetRequest(symbol: String, name: Option[String] = None, assetType: Option[String] = None, sector: Option[String] = None) {
  def toAsset(portfolioId: Int): Asset = Asset(None, portfolioId, symbol, name, assetType, sector, "USD")
}

case class CreateTransactionRequest(
  `type`: String,
  quantity: Double,
  price: Double,
  fee: Double = 0.0,
  `date`: String
) {
  def toTransaction(assetId: Int): Transaction = Transaction(None, assetId, `type`, quantity, price, fee, `date`)
}
