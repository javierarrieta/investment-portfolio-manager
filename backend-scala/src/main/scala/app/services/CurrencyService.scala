package app.services

import cats.effect.IO
import cats.effect.Ref
import cats.effect.Resource
import cats.implicits._
import doobie._
import doobie.hikari.HikariTransactor
import doobie.implicits._
import doobie.implicits.javatimedrivernative._
import doobie.implicits.toSqlInterpolator
import doobie.implicits.toConnectionIOOps
import io.circe.Parser
import org.http4s.ember.client.EmberClientBuilder
import org.http4s.client.Client
import org.http4s.Uri

case class ExchangeRate(symbol: String, rate: Double)

case class CurrencyService(
  xa: HikariTransactor[IO],
  cache: Ref[IO, Map[String, (Double, Long)]],
  httpClient: Resource[IO, Client[IO]]
) {
  import CurrencyService._

  def getExchangeRate(symbol: String): IO[Double] =
    cache.get.flatMap { cache =>
      cache.get(symbol) match {
        case Some((rate: Double, ts: Long)) if System.currentTimeMillis() - ts < cacheTtlMs =>
          IO.pure(rate)
        case Some(_) =>
          fetchExchangeRate(symbol)
        case None =>
          fetchExchangeRate(symbol)
      }
    }

  private def fetchExchangeRate(symbol: String): IO[Double] =
    if (symbol == "USD") IO.pure(1.0)
    else {
      val query = sql"SELECT rate FROM exchange_rates WHERE symbol = $symbol"
        .query[Double].option.transact(xa)
      query.flatMap {
        case Some(rate: Double) => IO.pure(rate)
        case None =>
          fetchFromYahoo(symbol).flatMap { rate =>
            val insert = sql"INSERT OR REPLACE INTO exchange_rates (symbol, rate, updated_at) VALUES ($symbol, $rate, datetime('now'))"
              .update.run.transact(xa)
            insert >> cache.update { cache =>
              cache.updated(symbol, (rate, System.currentTimeMillis()))
            } >> IO.pure(rate)
          }
      }
    }

  private def fetchFromYahoo(symbol: String): IO[Double] = {
    val fromBase = symbol.dropRight(2)
    val toBase = "USD"
    val url = Uri.fromString(s"https://query1.finance.yahoo.com/v7/finance/quote?symbols=${fromBase}${toBase}=X")
    url match {
      case Right(uri) =>
        httpClient.use { client =>
          for {
            response <- client.expect[String](uri)
            result <- IO.fromEither(io.circe.parser.parse(response)).flatMap { json =>
              IO.fromEither(
                json.hcursor.downField("quoteResponse").downField("result").downN(0)
                  .downField("regularMarketPrice").as[Double]
              )
            }
          } yield result
        }
      case Left(_) => IO.pure(1.0)
    }
  }

  def convertToUsd(amount: Double, currency: String): IO[Double] =
    if (currency == "USD") IO.pure(amount)
    else getExchangeRate(currency).map(amount * _)

  def convertFromUsd(amount: Double, currency: String): IO[Double] =
    if (currency == "USD") IO.pure(amount)
    else getExchangeRate(currency).map(amount / _)

  def convertBetween(amount: Double, from: String, to: String): IO[Double] =
    if (from == to) IO.pure(amount)
    else {
      for {
        usdAmount <- convertFromUsd(amount, from)
        result <- convertToUsd(usdAmount, to)
      } yield result
    }

  def refreshExchangeRate(symbol: String): IO[Double] =
    for {
      rate <- fetchExchangeRate(symbol)
      _ <- cache.update { cache =>
        cache.updated(symbol, (rate, System.currentTimeMillis()))
      }
    } yield rate

  def getCacheTimestamps: IO[Map[String, Long]] =
    cache.get.map {
      _.map { case (k, (_, ts)) => k -> ts }
    }
}

object CurrencyService {

  val cacheTtlMs = 300000L

  def create(xa: HikariTransactor[IO]): Resource[IO, CurrencyService] = {
    val cacheResource = Resource.eval(Ref.of[IO, Map[String, (Double, Long)]](Map.empty))
    val httpClientResource = EmberClientBuilder.default[IO].build
    (cacheResource, httpClientResource).mapN { (cache, httpClient) =>
      CurrencyService(xa, cache, Resource.pure(httpClient))
    }
  }
}
