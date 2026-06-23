package app.models

import java.time.{LocalDate, LocalDateTime}
import io.circe.Codec
import io.circe.generic.semiauto._
import io.circe.syntax._
import io.circe._

case class Portfolio(
  id: Option[Int],
  name: String,
  description: Option[String],
  currency: String,
  baseCurrency: String
) derives Codec.AsObject

object Portfolio {
  implicit val encoder: Encoder[Portfolio] = deriveEncoder[Portfolio]
}

case class Asset(
  id: Option[Int],
  portfolioId: Int,
  symbol: String,
  name: Option[String],
  assetType: Option[String],
  sector: Option[String],
  currency: String
) derives Codec.AsObject

object Asset {
  implicit val encoder: Encoder[Asset] = deriveEncoder[Asset]
}

case class Transaction(
  id: Option[Int],
  assetId: Int,
  `type`: String,
  quantity: Double,
  price: Double,
  fee: Double,
  `date`: String
) derives Codec.AsObject

object Transaction {
  implicit val encoder: Encoder[Transaction] = deriveEncoder[Transaction]
}

case class HistoricalPrice(
  symbol: String,
  date: LocalDate,
  closePrice: Double
) derives Codec.AsObject

case class TaxLot(
  assetId: Int,
  lotId: Int,
  purchaseDate: LocalDate,
  quantity: Double,
  purchasePrice: Double,
  totalCost: Double
) derives Codec.AsObject

case class PortfolioSummary(
  portfolioId: Int,
  totalValue: Double,
  totalGain: Double,
  totalGainPercent: Double,
  assets: List[AssetSummary]
) derives Codec.AsObject

case class AssetSummary(
  symbol: String,
  quantity: Double,
  costBasis: Double,
  currentValue: Double,
  gain: Double,
  gainPercent: Double
) derives Codec.AsObject

case class StatsResult(
  startDate: String,
  endDate: String,
  timeWeightedReturn: Double,
  annualizedReturn: Double,
  volatility: Double,
  sharpeRatio: Double,
  benchmarkReturn: Double,
  beta: Double,
  alpha: Double
) derives Codec.AsObject

object StatsResult {
  implicit val encoder: Encoder[StatsResult] = deriveEncoder[StatsResult]
}

// API response schemas matching Python OpenAPI spec
case class PortfolioOut(
  name: String,
  description: Option[String],
  currency: String,
  id: Int,
  assets: List[AssetOut]
)

object PortfolioOut {
  implicit val encoder: Encoder[PortfolioOut] = Encoder.forProduct4("name", "description", "currency", "id")(p => (p.name, p.description, p.currency, p.id))
}

case class AssetOut(
  symbol: String,
  name: Option[String],
  assetType: Option[String],
  sector: Option[String],
  id: Int,
  portfolioId: Int,
  transactions: List[TransactionOut]
)

object AssetOut {
  implicit val encoder: Encoder[AssetOut] = Encoder.forProduct7("symbol", "name", "asset_type", "sector", "id", "portfolio_id", "transactions")(a => (a.symbol, a.name, a.assetType, a.sector, a.id, a.portfolioId, a.transactions))
  implicit val transactionOutEncoder: Encoder[TransactionOut] = Encoder.forProduct7("type", "quantity", "price", "fee", "date", "id", "asset_id")(t => (t.`type`, t.quantity, t.price, t.fee, t.`date`, t.id, t.assetId))
}

case class TransactionOut(
  `type`: String,
  quantity: Double,
  price: Double,
  fee: Double,
  `date`: String,
  id: Int,
  assetId: Int
)

object ValidationError {
  implicit val encoder: Encoder[ValidationError] = Encoder.forProduct3("loc", "msg", "type")(e => (e.loc, e.msg, e.`type`))
}

case class ValidationError(
  loc: List[String],
  msg: String,
  `type`: String
)

case class HTTPValidationError(
  detail: List[ValidationError]
)

object HTTPValidationError {
  implicit val encoder: Encoder[HTTPValidationError] = Encoder.forProduct1("detail")(e => List(e.detail))
}
