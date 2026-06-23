package app.db

import app.models
import app.models._
import cats.effect.IO
import cats.effect.Async
import cats.data.NonEmptyList
import cats.syntax.all._
import doobie._
import doobie.implicits._
import doobie.implicits.javatimedrivernative._
import doobie.hikari.HikariTransactor
import java.time.LocalDate

class Repository[F[_]](xa: HikariTransactor[F])(implicit F: Async[F]) {

  def listPortfolios(): F[List[Portfolio]] =
    sql"SELECT id, name, description, currency, base_currency FROM portfolios"
      .query[Portfolio].stream.transact(xa).compile.toList

  def getPortfolio(id: Int): F[Option[Portfolio]] =
    sql"SELECT id, name, description, currency, base_currency FROM portfolios WHERE id = $id"
      .query[Portfolio].option.transact(xa)

  def createPortfolio(p: Portfolio): F[Portfolio] =
    sql"INSERT INTO portfolios (name, description, currency, base_currency) VALUES (${p.name}, ${p.description}, ${p.currency}, ${p.baseCurrency})"
      .update.withUniqueGeneratedKeys[Int]("id")
      .map(id => p.copy(id = Some(id))).transact(xa)

  def deletePortfolio(id: Int): F[Boolean] =
    sql"DELETE FROM portfolios WHERE id = $id"
      .update.run.transact(xa).map(_ > 0)

  def updatePortfolio(id: Int, name: String): F[Option[Portfolio]] =
    sql"UPDATE portfolios SET name = $name WHERE id = $id"
      .update.run.transact(xa).flatMap {
        case 1 => getPortfolio(id)
        case _ => F.pure(None)
      }

  def listAssets(portfolioId: Int): F[List[Asset]] =
    sql"SELECT id, portfolio_id, symbol, name, asset_type, sector, currency FROM assets WHERE portfolio_id = $portfolioId"
      .query[Asset].stream.transact(xa).compile.toList

  def listTransactions(assetId: Int): F[List[Transaction]] =
    sql"SELECT id, asset_id, \`type\`, quantity, price, fee, \`date\` FROM transactions WHERE asset_id = $assetId"
      .query[Transaction].stream.transact(xa).compile.toList

  def listTransactionsByPortfolio(portfolioId: Int): F[List[Transaction]] =
    sql"SELECT t.id, t.asset_id, t.\`type\`, t.quantity, t.price, t.fee, t.\`date\` FROM transactions t JOIN assets a ON t.asset_id = a.id WHERE a.portfolio_id = $portfolioId"
      .query[Transaction].stream.transact(xa).compile.toList

  def getTransaction(id: Int): F[Option[Transaction]] =
    sql"SELECT id, asset_id, \`type\`, quantity, price, fee, \`date\` FROM transactions WHERE id = $id"
      .query[Transaction].option.transact(xa)

  def createAsset(p: Asset): F[Asset] =
    sql"INSERT INTO assets (portfolio_id, symbol, name, asset_type, sector, currency) VALUES (${p.portfolioId}, ${p.symbol}, ${p.name}, ${p.assetType}, ${p.sector}, ${p.currency})"
      .update.withUniqueGeneratedKeys[Int]("id")
      .map(id => p.copy(id = Some(id))).transact(xa)

  def deleteAsset(id: Int): F[Boolean] =
    sql"DELETE FROM assets WHERE id = $id"
      .update.run.transact(xa).map(_ > 0)

  def updateAsset(p: Asset): F[Option[Asset]] =
    sql"UPDATE assets SET portfolio_id = ${p.portfolioId}, symbol = ${p.symbol}, name = ${p.name}, asset_type = ${p.assetType}, sector = ${p.sector}, currency = ${p.currency} WHERE id = ${p.id.getOrElse(0)}"
      .update.run.transact(xa).flatMap {
        case 1 => getAsset(p.id.getOrElse(0))
        case _ => F.pure(None)
      }

  def getAsset(id: Int): F[Option[Asset]] =
    sql"SELECT id, portfolio_id, symbol, name, asset_type, sector, currency FROM assets WHERE id = $id"
      .query[Asset].option.transact(xa)

  def insertHistoricalPrices(prices: List[HistoricalPrice]): F[Int] = {
    val sql = "INSERT OR REPLACE INTO historical_prices (symbol, date, close_price) VALUES (?, ?, ?)"
    Update[HistoricalPrice](sql).updateMany(prices).transact(xa)
  }

  def getHistoricalPrices(symbols: List[String], start: LocalDate, end: LocalDate): F[List[HistoricalPrice]] = {
    NonEmptyList.fromList(symbols) match {
      case None => F.pure(Nil)
      case Some(syms) =>
        val symsFragments = syms.map(s => fr"$s")
        val symsConcat = Fragments.comma(symsFragments)
        val q = fr"symbol IN (" ++ symsConcat ++ fr")"
        (fr"SELECT symbol, date, close_price FROM historical_prices WHERE" ++ q ++ fr"AND date >= $start AND date <= $end")
          .query[HistoricalPrice].stream.transact(xa).compile.toList
    }
  }

  def getHistoricalPricesByTicker(portfolioId: Int, ticker: String, start: LocalDate, end: LocalDate): F[List[HistoricalPrice]] =
    (fr"SELECT symbol, date, close_price FROM historical_prices WHERE symbol = $ticker AND date >= $start AND date <= $end")
      .query[HistoricalPrice].stream.transact(xa).compile.toList

  def createTransaction(t: Transaction): F[Transaction] =
    sql"INSERT INTO transactions (asset_id, \`type\`, quantity, price, fee, \`date\`) VALUES (${t.assetId}, ${t.`type`}, ${t.quantity}, ${t.price}, ${t.fee}, ${t.`date`})"
      .update.withUniqueGeneratedKeys[Int]("id")
      .map(id => t.copy(id = Some(id))).transact(xa)

  def deleteTransaction(id: Int): F[Boolean] =
    sql"DELETE FROM transactions WHERE id = $id"
      .update.run.transact(xa).map(_ > 0)
}

object Repository {
  def apply[F[_]: Async](xa: HikariTransactor[F]): Repository[F] = new Repository(xa)
}
