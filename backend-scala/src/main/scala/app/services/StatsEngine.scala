package app.services

import app.models._
import cats.effect.IO
import cats.effect.Resource
import cats.implicits._
import doobie._
import doobie.hikari.HikariTransactor
import doobie.implicits._
import doobie.implicits.javatimedrivernative._
import doobie.implicits.toSqlInterpolator
import doobie.implicits.toConnectionIOOps
import java.time.LocalDate
import io.circe.syntax._
import io.circe.Encoder
import io.circe.Codec
import io.circe.Json
import io.circe.generic.semiauto._

object StatsEngine {

  def create(xa: HikariTransactor[IO]): StatsEngine = new StatsEngine(xa)
}

case class PortfolioStats(
  portfolioId: Int,
  timeWeightedReturn: Double,
  volatility: Double,
  sharpeRatio: Double,
  beta: Double
)

case class DailyReturn(
  date: LocalDate,
  dailyReturn: Double
)

case class BenchmarkReturn(
  date: LocalDate,
  benchmarkReturn: Double
)

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

object StatsEncoder {
  implicit val portfolioStatsEncoder: Encoder[PortfolioStats] = deriveEncoder[PortfolioStats]
}

class StatsEngine(xa: HikariTransactor[IO]) {

  def calculatePortfolioStats(portfolioId: Int): IO[PortfolioStats] = {
    calculateTimeWeightedReturn(portfolioId).flatMap { twr =>
      calculateVolatility(portfolioId).flatMap { volatility =>
        calculateSharpeRatio(twr, volatility).flatMap { sharpe =>
          calculateBeta(portfolioId).flatMap { beta =>
            IO.pure(PortfolioStats(portfolioId, twr, volatility, sharpe, beta))
          }
        }
      }
    }
  }

  def calculateTimeWeightedReturn(portfolioId: Int): IO[Double] = {
    val query = fr"""
      SELECT date, close_price FROM historical_prices
      WHERE symbol IN (
        SELECT symbol FROM assets WHERE portfolio_id = $portfolioId
      )
      ORDER BY date
    """
    query.query[(LocalDate, Double)].to[List].transact(xa).flatMap { prices =>
      val dailyReturns = calculateDailyReturns(prices)
      val cumulativeReturn = dailyReturns.foldLeft(1.0) { (acc, r) =>
        acc * (1.0 + r)
      }
      IO.pure(cumulativeReturn - 1.0)
    }
  }

  def calculateVolatility(portfolioId: Int): IO[Double] = {
    val query = fr"""
      SELECT date, close_price FROM historical_prices
      WHERE symbol IN (
        SELECT symbol FROM assets WHERE portfolio_id = $portfolioId
      )
      ORDER BY date
    """
    query.query[(LocalDate, Double)].to[List].transact(xa).flatMap { prices =>
      val dailyReturns = calculateDailyReturns(prices)
      val mean = dailyReturns.sum / dailyReturns.size
      val variance = dailyReturns.map(r => Math.pow(r - mean, 2)).sum / (dailyReturns.size - 1)
      IO.pure(Math.sqrt(variance))
    }
  }

  def calculateSharpeRatio(twr: Double, volatility: Double): IO[Double] = {
    if (volatility == 0) IO.pure(0.0)
    else {
      val annualizedReturn = calculateAnnualizedReturn(twr, 365)
      val riskFreeRate = 0.04
      IO.pure((annualizedReturn - riskFreeRate) / volatility)
    }
  }

  def calculateBeta(portfolioId: Int): IO[Double] = {
    val query = fr"""
      SELECT date, close_price FROM historical_prices
      WHERE symbol IN (
        SELECT symbol FROM assets WHERE portfolio_id = $portfolioId
      )
      ORDER BY date
    """
    val spyQuery = fr"SELECT date, close_price FROM historical_prices WHERE symbol = 'SPY' ORDER BY date"
    for {
      portfolioPrices <- query.query[(LocalDate, Double)].to[List].transact(xa)
      spyPrices <- spyQuery.query[(LocalDate, Double)].to[List].transact(xa)
    } yield calculateBetaFromPrices(portfolioPrices, spyPrices)
  }

  def calculateAnnualizedReturn(twr: Double, days: Int): Double = {
    Math.pow(1.0 + twr, 365.0 / days) - 1.0
  }

  def calculateDailyReturns(prices: List[(LocalDate, Double)]): List[Double] = {
    prices.sliding(2).collect {
      case List((_, prev), (_, curr)) if prev > 0 =>
        (curr - prev) / prev
    }.toList
  }

  def calculateBetaFromPrices(portfolioPrices: List[(LocalDate, Double)], spyPrices: List[(LocalDate, Double)]): Double = {
    val portfolioReturns = calculateDailyReturns(portfolioPrices)
    val spyReturns = calculateDailyReturns(spyPrices)
    val minLen = Math.min(portfolioReturns.size, spyReturns.size)
    val pairedReturns = portfolioReturns.take(minLen).zip(spyReturns.take(minLen))

    if (pairedReturns.size < 2) 0.0
    else {
      val portfolioMean = pairedReturns.map(_._1).sum / pairedReturns.size
      val spyMean = pairedReturns.map(_._2).sum / pairedReturns.size
      val covariance = pairedReturns.map { case (pr, sr) =>
        (pr - portfolioMean) * (sr - spyMean)
      }.sum / (pairedReturns.size - 1)
      val spyVariance = pairedReturns.map(p => Math.pow(p._2 - spyMean, 2)).sum / (pairedReturns.size - 1)
      if (spyVariance == 0) 0.0
      else covariance / spyVariance
    }
  }

  def calculateStats(startDate: String, endDate: String): IO[StatsResult] = {
    val start = LocalDate.parse(startDate)
    val end = LocalDate.parse(endDate)

    val query = fr"""
      SELECT symbol, date, close_price FROM historical_prices
      WHERE date >= $start AND date <= $end
      ORDER BY date
    """
    query.query[(String, LocalDate, Double)].to[List].transact(xa).flatMap { prices =>
      val groupedBySymbol = prices.groupBy(_._1)
      val portfolioReturn = calculateTimeWeightedReturnFromPrices(groupedBySymbol.values.toList.map { pair =>
        pair.map(p => (p._2, p._3)).sortBy(_._1)
      })
      val spyReturn = groupedBySymbol.get("SPY").map { spyPrices =>
        calculateTimeWeightedReturnFromPrices(List(spyPrices.map(p => (p._2, p._3)).sortBy(_._1)))
      }.getOrElse(0.0)

      IO.pure(StatsResult(
        startDate,
        endDate,
        portfolioReturn,
        portfolioReturn * 365.0 / (end.toEpochDay - start.toEpochDay),
        0.0,
        0.0,
        spyReturn,
        1.0,
        portfolioReturn - spyReturn
      ))
    }
  }

  def calculateTimeWeightedReturnFromPrices(pricesLists: List[List[(LocalDate, Double)]]): Double = {
    val dailyReturns = pricesLists.flatMap { prices =>
      prices.sliding(2).collect {
        case List((_, prev), (_, curr)) if prev > 0 =>
          (curr - prev) / prev
      }
    }
    if (dailyReturns.isEmpty) 0.0
    else {
      val cumulativeReturn = dailyReturns.foldLeft(1.0) { (acc, r) =>
        acc * (1.0 + r)
      }
      cumulativeReturn - 1.0
    }
  }

  def calculateBenchmarkReturn(startDate: String, endDate: String): IO[Double] = {
    val start = LocalDate.parse(startDate)
    val end = LocalDate.parse(endDate)

    val query = fr"SELECT date, close_price FROM historical_prices WHERE symbol = 'SPY' AND date >= $start AND date <= $end ORDER BY date"
    query.query[(LocalDate, Double)].to[List].transact(xa).flatMap { prices =>
      IO.pure(calculateTimeWeightedReturnFromPrices(List(prices.sortBy(_._2))))
    }
  }

  def getHistoricalPrices(portfolioId: Int, symbol: String, startDate: LocalDate, endDate: LocalDate): IO[List[(LocalDate, Double)]] = {
    val query = fr"""
      SELECT date, close_price FROM historical_prices
      WHERE symbol = $symbol AND date >= $startDate AND date <= $endDate
      ORDER BY date
    """
    query.query[(LocalDate, Double)].to[List].transact(xa)
  }
}
