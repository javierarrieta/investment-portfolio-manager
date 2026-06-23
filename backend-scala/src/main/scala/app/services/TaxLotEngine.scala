package app.services

import app.models._
import cats.effect.IO
import cats.effect.Resource
import io.circe.syntax._
import io.circe.Encoder
import io.circe.Json
import doobie._
import doobie.hikari.HikariTransactor
import doobie.implicits._
import doobie.implicits.javatimedrivernative._
import doobie.implicits.toSqlInterpolator
import doobie.implicits.toConnectionIOOps

object TaxLotEngine {

  def create(xa: HikariTransactor[IO]): TaxLotEngine = new TaxLotEngine(xa)
}

case class TaxLot(
  id: Int,
  assetId: Int,
  purchaseDate: java.time.LocalDate,
  quantity: Double,
  remainingQuantity: Double,
  costBasis: Double,
  costBasisUsd: Double,
  priceUsd: Double,
  currency: String,
  strategy: String
)

case class LotDisposition(
  lotId: Int,
  quantity: Double
)

case class SaleResult(
  dispositions: List[LotDisposition],
  totalProceeds: Double,
  totalCostBasis: Double,
  capitalGain: Double
)

case class TaxLotsResponse(
  strategy: String,
  lots: List[TaxLotResponse]
)

object TaxLotsResponse {
  implicit val encoder: Encoder[TaxLotsResponse] = new Encoder[TaxLotsResponse] {
    def apply(r: TaxLotsResponse): Json = Json.obj(
      "strategy" -> Json.fromString(r.strategy),
      "lots" -> Encoder.encodeIterable[TaxLotResponse, List](TaxLotEncoder.taxLotResponseEncoder)(r.lots)
    )
  }
}

case class TaxLotResponse(
  assetId: Int,
  lotId: Int,
  purchaseDate: String,
  quantity: Double,
  remainingQuantity: Double,
  costBasis: Double,
  costBasisUsd: Double,
  priceUsd: Double,
  currency: String,
  strategy: String
)

case class SaleResponse(
  strategy: String,
  result: SaleResult,
  remainingLots: List[TaxLotResponse]
)

object TaxLotEncoder {
  implicit val taxLotEncoder: Encoder[TaxLot] = Encoder.forProduct10("id", "asset_id", "purchase_date", "quantity", "remaining_quantity", "cost_basis", "cost_basis_usd", "price_usd", "currency", "strategy")(l => (l.id, l.assetId, l.purchaseDate.toString, l.quantity, l.remainingQuantity, l.costBasis, l.costBasisUsd, l.priceUsd, l.currency, l.strategy))
  implicit val taxLotResponseEncoder: Encoder[TaxLotResponse] = Encoder.forProduct10("asset_id", "lot_id", "purchase_date", "quantity", "remaining_quantity", "cost_basis", "cost_basis_usd", "price_usd", "currency", "strategy")(l => (l.assetId, l.lotId, l.purchaseDate, l.quantity, l.remainingQuantity, l.costBasis, l.costBasisUsd, l.priceUsd, l.currency, l.strategy))
  implicit val taxLotsResponseEncoder: Encoder[TaxLotsResponse] = Encoder.forProduct2("strategy", "lots")(r => (r.strategy, r.lots))
}

class TaxLotEngine(xa: HikariTransactor[IO]) {

  def getTaxLots(strategy: String): IO[List[TaxLotResponse]] = {
    val query = fr"""
      SELECT id, asset_id, purchase_date, quantity, remaining_quantity, cost_basis, cost_basis_usd, price_usd, currency, $strategy as strategy
      FROM tax_lots
    """
    query.query[TaxLot].to[List].transact(xa).map { lots =>
      lots.map { lot =>
        TaxLotResponse(lot.assetId, lot.id, lot.purchaseDate.toString, lot.quantity, lot.remainingQuantity, lot.costBasis, lot.costBasisUsd, lot.priceUsd, lot.currency, lot.strategy)
      }
    }
  }

  def processSale(saleId: Int): IO[SaleResponse] = {
    val query = fr"""
      SELECT id, asset_id, purchase_date, quantity, remaining_quantity, cost_basis, cost_basis_usd, price_usd, currency, strategy
      FROM tax_lots
      WHERE lot_id = $saleId
    """
    query.query[TaxLot].option.transact(xa).flatMap {
      case Some(lot) =>
        val disposition = LotDisposition(lot.id, lot.remainingQuantity)
        val result = SaleResult(
          List(disposition),
          lot.priceUsd * lot.remainingQuantity,
          lot.costBasis * lot.remainingQuantity,
          (lot.priceUsd - lot.costBasis) * lot.remainingQuantity
        )
        val remainingLots = List.empty[TaxLotResponse]
        IO.pure(SaleResponse(lot.strategy, result, remainingLots))
      case None =>
        IO.raiseError(new RuntimeException(s"Tax lot not found: $saleId"))
    }
  }

  def disposeLot(lotId: Int, quantity: Double): IO[LotDisposition] = {
    val query = fr"""
      SELECT id, remaining_quantity, cost_basis, price_usd
      FROM tax_lots WHERE id = $lotId
    """
    query.query[(Int, Double, Double, Double)].option.transact(xa).flatMap {
      case Some((_, remaining, costBasis, priceUsd)) =>
        val disposed = quantity.min(remaining)
        val updateQuery = fr"UPDATE tax_lots SET remaining_quantity = remaining_quantity - $disposed WHERE id = $lotId"
        updateQuery.update.run.transact(xa).flatMap {
          case 1 => IO.pure(LotDisposition(lotId, disposed))
          case _ => IO.raiseError(new RuntimeException(s"Failed to dispose lot: $lotId"))
        }
      case None =>
        IO.raiseError(new RuntimeException(s"Tax lot not found: $lotId"))
    }
  }

  def processSaleByStrategy(saleId: Int, strategy: String): IO[SaleResponse] = {
    val query = fr"""
      SELECT id, asset_id, purchase_date, quantity, remaining_quantity, cost_basis, cost_basis_usd, price_usd, currency, strategy
      FROM tax_lots
      WHERE lot_id = $saleId
    """
    query.query[TaxLot].to[List].transact(xa).flatMap { lots =>
      strategy match {
        case "FIFO" =>
          val sorted = lots.sortBy(_.purchaseDate)
          processFIFO(saleId, sorted)
        case "LIFO" =>
          val sorted = lots.sortBy(_.purchaseDate).reverse
          processLIFO(saleId, sorted)
        case _ =>
          IO.raiseError(new RuntimeException(s"Unknown strategy: $strategy"))
      }
    }
  }

  private def processFIFO(saleId: Int, lots: List[TaxLot]): IO[SaleResponse] = {
    lots.foldLeft(IO.pure((List.empty[LotDisposition], 0.0, 0.0, 0.0))) { (acc, lot) =>
      acc.flatMap { (dispositions, proceeds, basis, gain) =>
        val disposed = lot.remainingQuantity
        val disposition = LotDisposition(lot.id, disposed)
        val newProceeds = proceeds + (lot.priceUsd * disposed)
        val newBasis = basis + (lot.costBasis * disposed)
        val newGain = gain + ((lot.priceUsd - lot.costBasis) * disposed)
        IO.pure((dispositions :+ disposition, newProceeds, newBasis, newGain))
      }
    }.map { (dispositions, proceeds, basis, gain) =>
      val result = SaleResult(dispositions, proceeds, basis, gain)
      SaleResponse("FIFO", result, List.empty[TaxLotResponse])
    }
  }

  private def processLIFO(saleId: Int, lots: List[TaxLot]): IO[SaleResponse] = {
    lots.foldLeft(IO.pure((List.empty[LotDisposition], 0.0, 0.0, 0.0))) { (acc, lot) =>
      acc.flatMap { (dispositions, proceeds, basis, gain) =>
        val disposed = lot.remainingQuantity
        val disposition = LotDisposition(lot.id, disposed)
        val newProceeds = proceeds + (lot.priceUsd * disposed)
        val newBasis = basis + (lot.costBasis * disposed)
        val newGain = gain + ((lot.priceUsd - lot.costBasis) * disposed)
        IO.pure((dispositions :+ disposition, newProceeds, newBasis, newGain))
      }
    }.map { (dispositions, proceeds, basis, gain) =>
      val result = SaleResult(dispositions, proceeds, basis, gain)
      SaleResponse("LIFO", result, List.empty[TaxLotResponse])
    }
  }

  def updateRemainingQuantity(lotId: Int, newRemaining: Double): IO[Int] = {
    val updateQuery = fr"UPDATE tax_lots SET remaining_quantity = $newRemaining WHERE id = $lotId"
    updateQuery.update.run.transact(xa)
  }

  def getTaxSummary(portfolioId: Int, strategy: String, thresholdDays: Int): IO[Json] = {
    val query = fr"""
      SELECT id, asset_id, purchase_date, quantity, remaining_quantity, cost_basis, cost_basis_usd, price_usd, currency, strategy
      FROM tax_lots
      WHERE asset_id IN (
        SELECT id FROM assets WHERE portfolio_id = $portfolioId
      )
      ORDER BY purchase_date
    """
    query.query[TaxLot].to[List].transact(xa).flatMap { lots =>
      val sortedLots = strategy.toUpperCase match {
        case "FIFO" => lots.sortBy(_.purchaseDate)
        case "LIFO" => lots.sortBy(_.purchaseDate).reverse
        case "HYBRID" =>
          val now = java.time.LocalDate.now()
          val (shortTerm, longTerm) = lots.partition(l => java.time.Duration.between(l.purchaseDate, now).toDays <= thresholdDays)
          shortTerm.sortBy(_.purchaseDate).reverse ++ longTerm.sortBy(_.purchaseDate)
        case _ => lots.sortBy(_.purchaseDate)
      }
      val response = TaxLotsResponse(strategy, sortedLots.map(l => TaxLotResponse(l.assetId, l.id, l.purchaseDate.toString, l.quantity, l.remainingQuantity, l.costBasis, l.costBasisUsd, l.priceUsd, l.currency, l.strategy)))
      IO.pure(response.asJson)
    }
  }
}
