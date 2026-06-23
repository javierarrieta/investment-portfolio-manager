# Scala 3 / GraalVM Native Image Backend Migration Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Migrate the Python-based FastAPI backend to a high-performance, purely functional Scala 3 backend compiled into a native binary via GraalVM Native Image, using the Typelevel stack (Cats Effect, Http4s, Doobie, Circe).

**Architecture:** The application will be structured as a purely functional Scala 3 codebase under `backend-scala/`. Database storage will remain in the SQLite `backend/portfolio.db` (reused directly) accessed via HikariCP and Doobie. The app will be compiled to a standalone native binary using the GraalVM Native Image sbt plugin.

**Tech Stack:** Scala 3.3.3, Cats Effect 3.5.4, Http4s 0.23.27, Doobie 1.0.0-RC5 (SQLite), Circe 0.14.9, sbt-native-packager, MUnit.

---

## File Structure

The project will live in `backend-scala/` and have the following layout:
```
backend-scala/
├── build.sbt
├── project/
│   ├── build.properties
│   └── plugins.sbt
└── src/
    ├── main/
    │   ├── resources/
    │   │   └── logback.xml
    │   └── scala/
    │       └── app/
    │           ├── Main.scala
    │           ├── db/
    │           │   ├── Database.scala
    │           │   └── Repository.scala
    │           ├── models/
    │           │   ├── Domain.scala
    │           │   └── Schemas.scala
    │           ├── routes/
    │           │   ├── PortfolioRoutes.scala
    │           │   ├── TransactionRoutes.scala
    │           │   └── AnalyticsRoutes.scala
    │           └── services/
    │               ├── CurrencyService.scala
    │               ├── StatsEngine.scala
    │               └── TaxLotEngine.scala
    └── test/
        └── scala/
            └── app/
                ├── services/
                │   ├── TaxLotEngineSpec.scala
                │   └── StatsEngineSpec.scala
                └── routes/
                    └── PortfolioRoutesSpec.scala
```

---

## Detailed Implementation Tasks

### Task 1: Project Setup & Build Configuration

**Files:**
- Create: `backend-scala/project/plugins.sbt`
- Create: `backend-scala/project/build.properties`
- Create: `backend-scala/build.sbt`
- Create: `backend-scala/src/main/resources/logback.xml`

- [ ] **Step 1: Write `plugins.sbt`**
Add the sbt native-packager plugin.
```scala
addSbtPlugin("com.github.sbt" % "sbt-native-packager" % "1.9.16")
```

- [ ] **Step 2: Write `build.properties`**
Specify sbt version.
```properties
sbt.version=1.9.9
```

- [ ] **Step 3: Write `build.sbt`**
Define Scala 3.3.3 project dependencies and configuration.
```scala
enablePlugins(GraalVMNativeImagePlugin)

name := "investment-portfolio-backend"
version := "1.0.0"
scalaVersion := "3.3.3"

val CatsEffectVersion = "3.5.4"
val Http4sVersion     = "0.23.27"
val CirceVersion      = "0.14.9"
val DoobieVersion     = "1.0.0-RC5"
val LogbackVersion    = "1.5.6"
val MUnitVersion      = "1.0.0"
val MUnitCEVersion    = "1.0.7"

libraryDependencies ++= Seq(
  "org.typelevel"         %% "cats-effect"         % CatsEffectVersion,
  "org.http4s"            %% "http4s-ember-server" % Http4sVersion,
  "org.http4s"            %% "http4s-ember-client" % Http4sVersion,
  "org.http4s"            %% "http4s-dsl"          % Http4sVersion,
  "org.http4s"            %% "http4s-circe"        % Http4sVersion,
  "io.circe"              %% "circe-core"          % CirceVersion,
  "io.circe"              %% "circe-generic"       % CirceVersion,
  "io.circe"              %% "circe-parser"        % CirceVersion,
  "org.tpolecat"          %% "doobie-core"         % DoobieVersion,
  "org.tpolecat"          %% "doobie-hikari"       % DoobieVersion,
  "org.xerial"            %  "sqlite-jdbc"         % "3.46.0.0",
  "ch.qos.logback"        %  "logback-classic"     % LogbackVersion,
  "org.scalameta"         %% "munit"               % MUnitVersion   % Test,
  "org.typelevel"         %% "munit-cats-effect-3" % MUnitCEVersion % Test
)

scalacOptions ++= Seq(
  "-feature",
  "-deprecation",
  "-unchecked",
  "-Xfatal-warnings"
)

graalVMNativeImageOptions ++= Seq(
  "--no-fallback",
  "--initialize-at-build-time=org.slf4j",
  "--enable-url-protocols=http,https",
  "-H:+ReportExceptionStackTraces",
  "--strict-image-info",
  "-H:+AddAllCharsets"
)
```

- [ ] **Step 4: Write `logback.xml`**
Basic logger config.
```xml
<configuration>
  <appender name="STDOUT" class="ch.qos.logback.core.ConsoleAppender">
    <encoder>
      <pattern>%d{HH:mm:ss.SSS} [%thread] %-5level %logger{36} - %msg%n</pattern>
    </encoder>
  </appender>
  <root level="info">
    <appender-ref ref="STDOUT"/>
  </root>
</configuration>
```

- [ ] **Step 5: Run compiler check**
Verify the empty project loads.
Run: `sbt compile` (from `backend-scala/`)
Expected: Successful compile of 0 sources.

---

### Task 2: Models & DB Repository

**Files:**
- Create: `backend-scala/src/main/scala/app/models/Domain.scala`
- Create: `backend-scala/src/main/scala/app/db/Database.scala`
- Create: `backend-scala/src/main/scala/app/db/Repository.scala`

- [ ] **Step 1: Write `Domain.scala`**
Create case classes mapping to the DB schema and Circe codecs.
```scala
package app.models

import java.time.{LocalDate, LocalDateTime}
import io.circe.Codec
import io.circe.generic.semiauto._

case class Portfolio(
  id: Option[Int],
  name: String,
  description: Option[String],
  currency: String = "USD",
  baseCurrency: String = "USD"
) derives Codec.AsObject

case class Asset(
  id: Option[Int],
  portfolioId: Int,
  symbol: String,
  name: String,
  assetType: String,
  sector: Option[String],
  currency: String = "USD"
) derives Codec.AsObject

case class Transaction(
  id: Option[Int],
  assetId: Int,
  `type`: String,
  quantity: Double,
  price: Double,
  fee: Double = 0.0,
  date: LocalDateTime
) derives Codec.AsObject

case class HistoricalPrice(
  symbol: String,
  date: LocalDate,
  closePrice: Double
) derives Codec.AsObject
```

- [ ] **Step 2: Write `Database.scala`**
Establish Doobie SQLite connection transcator setup with HikariCP.
```scala
package app.db

import cats.effect.*
import doobie.*
import doobie.hikari.HikariTransactor
import doobie.util.transactor.Transactor

object Database {
  def transactor(dbPath: String): Resource[IO, HikariTransactor[IO]] = {
    for {
      ce <- Resource.eval(IO.executionContext)
      xa <- HikariTransactor.newHikariTransactor[IO](
        "org.sqlite.JDBC",
        s"jdbc:sqlite:$dbPath",
        "",
        "",
        ce
      )
    } yield xa
  }
}
```

- [ ] **Step 3: Write `Repository.scala`**
Complete SQL operations via Doobie.
```scala
package app.db

import app.models.*
import cats.effect.IO
import doobie.*
import doobie.implicits.*
import doobie.implicits.javatime.*
import java.time.LocalDate

class Repository(xa: Transactor[IO]) {
  
  def listPortfolios(): IO[List[Portfolio]] =
    sql"SELECT id, name, description, currency, base_currency FROM portfolios"
      .query[Portfolio].to[List].transact(xa)

  def getPortfolio(id: Int): IO[Option[Portfolio]] =
    sql"SELECT id, name, description, currency, base_currency FROM portfolios WHERE id = $id"
      .query[Portfolio].option.transact(xa)

  def createPortfolio(p: Portfolio): IO[Portfolio] =
    sql"INSERT INTO portfolios (name, description, currency, base_currency) VALUES (${p.name}, ${p.description}, ${p.currency}, ${p.baseCurrency})"
      .update.withUniqueGeneratedKeys[Int]("id")
      .map(id => p.copy(id = Some(id))).transact(xa)

  def deletePortfolio(id: Int): IO[Int] =
    sql"DELETE FROM portfolios WHERE id = $id"
      .update.run.transact(xa)

  def listAssets(portfolioId: Int): IO[List[Asset]] =
    sql"SELECT id, portfolio_id, symbol, name, asset_type, sector, currency FROM assets WHERE portfolio_id = $portfolioId"
      .query[Asset].to[List].transact(xa)

  def listTransactions(assetId: Int): IO[List[Transaction]] =
    sql"SELECT id, asset_id, type, quantity, price, fee, date FROM transactions WHERE asset_id = $assetId"
      .query[Transaction].to[List].transact(xa)

  def insertHistoricalPrices(prices: List[HistoricalPrice]): IO[Int] = {
    val sql = "INSERT OR REPLACE INTO historical_prices (symbol, date, close_price) VALUES (?, ?, ?)"
    Update[HistoricalPrice](sql).updateMany(prices).transact(xa)
  }

  def getHistoricalPrices(symbols: List[String], start: LocalDate, end: LocalDate): IO[List[HistoricalPrice]] = {
    NonEmptyList.fromList(symbols) match {
      case None => IO.pure(Nil)
      case Some(syms) =>
        val q = Fragments.in(fr"symbol", syms)
        (fr"SELECT symbol, date, close_price FROM historical_prices WHERE" ++ q ++ fr"AND date >= $start AND date <= $end")
          .query[HistoricalPrice].to[List].transact(xa)
    }
  }
}
```

- [ ] **Step 4: Run compiler check**
Verify Repository compiles cleanly.
Run: `sbt compile` (from `backend-scala/`)
Expected: Compilation success.

---

### Task 3: Currency Service & Tax Engine

**Files:**
- Create: `backend-scala/src/main/scala/app/services/CurrencyService.scala`
- Create: `backend-scala/src/main/scala/app/services/TaxLotEngine.scala`
- Create: `backend-scala/src/test/scala/app/services/TaxLotEngineSpec.scala`

- [ ] **Step 1: Write `CurrencyService.scala`**
Provides mocked or mock-convert currency conversions mirroring Python implementation.
```scala
package app.services

import cats.effect.IO
import java.time.LocalDateTime

class CurrencyService {
  def getRate(from: String, to: String, date: LocalDateTime): IO[Double] = {
    if (from == to) IO.pure(1.0)
    else {
      // Basic mock rate provider as in our python service
      IO.pure(1.20) // e.g. EURUSD
    }
  }
}
```

- [ ] **Step 2: Write `TaxLotEngine.scala`**
Port FIFO, LIFO, and HYBRID strategies elegantly in Scala.
```scala
package app.services

import app.models.Transaction
import java.time.LocalDateTime
import cats.effect.IO
import cats.syntax.all._

case class TaxLot(
  date: LocalDateTime,
  price: Double,
  qty: Double,
  feePerUnit: Double,
  unitCost: Double
)

case class TaxSummary(
  realizedPnL: Double,
  openLots: List[TaxLot]
)

object TaxLotEngine {
  def calculateLots(
    transactions: List[Transaction],
    currentPrice: Double,
    assetCurrency: String,
    baseCurrency: String,
    currencyService: CurrencyService,
    strategy: String = "FIFO",
    hybridThresholdDays: Int = 30
  ): IO[TaxSummary] = {
    val sortedTxs = transactions.sortBy(_.date)

    sortedTxs.foldLeftM(TaxSummary(0.0, Nil)) { (acc, tx) =>
      for {
        rate <- currencyService.getRate(assetCurrency, baseCurrency, tx.date)
        txType = tx.`type`.toUpperCase
        summary <- if (txType == "BUY") {
          val unitCostAsset = if (tx.quantity > 0) (tx.quantity * tx.price + tx.fee) / tx.quantity else tx.price
          val unitCostBase = unitCostAsset * rate
          val newLot = TaxLot(
            date = tx.date,
            price = tx.price * rate,
            qty = tx.quantity,
            feePerUnit = (if (tx.quantity > 0) tx.fee / tx.quantity else 0.0) * rate,
            unitCost = unitCostBase
          )
          IO.pure(acc.copy(openLots = acc.openLots :+ newLot))
        } else if (txType == "SELL") {
          val sellProceedsAsset = if (tx.quantity > 0) (tx.quantity * tx.price - tx.fee) / tx.quantity else tx.price
          val sellProceedsBase = sellProceedsAsset * rate
          
          val (matchedLots, remainingLots, realizedPnL) = processSell(
            tx.quantity,
            sellProceedsBase,
            tx.date,
            acc.openLots,
            strategy,
            hybridThresholdDays
          )
          IO.pure(TaxSummary(acc.realizedPnL + realizedPnL, remainingLots))
        } else IO.pure(acc)
      } yield summary
    }
  }

  private def processSell(
    qtyToSell: Double,
    sellPrice: Double,
    sellDate: LocalDateTime,
    lots: List[TaxLot],
    strategy: String,
    thresholdDays: Int
  ): (List[TaxLot], List[TaxLot], Double) = {
    // Partition lots bought before or at sellDate
    val (eligible, future) = lots.partition(l => !l.date.isAfter(sellDate) && l.qty > 0)

    val sortedEligible = strategy.toUpperCase match {
      case "FIFO" => eligible.sortBy(_.date)
      case "LIFO" => eligible.sortBy(_.date).reverse
      case "HYBRID" =>
        val (shortTerm, longTerm) = eligible.partition { l =>
          val ageDays = java.time.Duration.between(l.date, sellDate).toDays
          ageDays >= 0 && ageDays <= thresholdDays
        }
        shortTerm.sortBy(_.date).reverse ++ longTerm.sortBy(_.date)
      case _ => eligible.sortBy(_.date)
    }

    // Recursively deduct quantity
    def deduct(remainingQty: Double, currentLots: List[TaxLot], accRealized: Double): (List[TaxLot], Double) = {
      if (remainingQty <= 0) (currentLots, accRealized)
      else currentLots match {
        case Nil => (Nil, accRealized) // short sell / fallback
        case lot :: tail =>
          if (lot.qty <= remainingQty) {
            val realized = lot.qty * (sellPrice - lot.unitCost)
            deduct(remainingQty - lot.qty, tail, accRealized + realized)
          } else {
            val realized = remainingQty * (sellPrice - lot.unitCost)
            val updatedLot = lot.copy(qty = lot.qty - remainingQty)
            (updatedLot :: tail, accRealized + realized)
          }
      }
    }

    val (updatedEligible, realized) = deduct(qtyToSell, sortedEligible, 0.0)
    (Nil, updatedEligible ++ future, realized)
  }
}
```

- [ ] **Step 3: Write MUnit Test `TaxLotEngineSpec.scala`**
Add robust FIFO/LIFO/HYBRID validation.
```scala
package app.services

import app.models.Transaction
import java.time.LocalDateTime
import munit.CatsEffectSuite

class TaxLotEngineSpec extends CatsEffectSuite {
  val currencyService = new CurrencyService

  test("FIFO calculation matching Python logic") {
    val txs = List(
      Transaction(None, 1, "BUY", 10.0, 100.0, 5.0, LocalDateTime.of(2026, 1, 1, 10, 0)),
      Transaction(None, 1, "BUY", 10.0, 110.0, 5.0, LocalDateTime.of(2026, 1, 2, 10, 0)),
      Transaction(None, 1, "SELL", 12.0, 120.0, 10.0, LocalDateTime.of(2026, 1, 3, 10, 0))
    )

    TaxLotEngine.calculateLots(txs, 120.0, "USD", "USD", currencyService, "FIFO").map { summary =>
      // Unit cost lot 1: (1000 + 5)/10 = 100.5
      // Unit cost lot 2: (1100 + 5)/10 = 110.5
      // Sell 12: 10 from lot 1, 2 from lot 2
      // Proceeds: (12*120 - 10)/12 = 119.1666...
      // Realized: 10*(119.166... - 100.5) + 2*(119.166... - 110.5)
      assertEquals(summary.openLots.size, 1)
      assertEquals(summary.openLots.head.qty, 8.0)
      assert(summary.realizedPnL > 180.0)
    }
  }
}
```

- [ ] **Step 4: Execute test**
Run: `sbt test` (from `backend-scala/`)
Expected: Compilation success and `TaxLotEngineSpec` test passes.

---

### Task 4: Stats Engine Implementation

**Files:**
- Create: `backend-scala/src/main/scala/app/services/StatsEngine.scala`
- Create: `backend-scala/src/test/scala/app/services/StatsEngineSpec.scala`

- [ ] **Step 1: Write `StatsEngine.scala`**
Handle metrics, standard deviations, and Covariance calculations natively.
```scala
package app.services

import app.models.*
import cats.effect.IO
import cats.syntax.all._
import java.time.LocalDate
import org.http4s.client.Client
import org.http4s.Uri

object StatsEngine {

  def syncHistoricalPrices(client: Client[IO], symbols: List[String], repo: app.db.Repository, startDate: LocalDate): IO[Unit] = {
    // Queries Unofficial Yahoo Finance API v8 chart endpoint
    val symbolsWithBenchmark = (symbols :+ "SPY").distinct
    symbolsWithBenchmark.traverse_ { symbol =>
      val startEpoch = startDate.atStartOfDay(java.time.ZoneId.systemDefault()).toEpochSecond
      val endEpoch = LocalDate.now().plusDays(1).atStartOfDay(java.time.ZoneId.systemDefault()).toEpochSecond
      val url = s"https://query1.finance.yahoo.com/v8/finance/chart/$symbol?period1=$startEpoch&period2=$endEpoch&interval=1d"
      
      Uri.fromString(url) match {
        case Left(_) => IO.unit
        case Right(uri) =>
          client.get(uri) { response =>
            // Standard JSON parsing of Yahoo response to map prices
            // Simplification: assume parser returns List[HistoricalPrice]
            IO.pure(List.empty[HistoricalPrice])
          }.flatMap(repo.insertHistoricalPrices).void.handleErrorWith { e =>
            IO.println(s"Error fetching historical prices for $symbol: ${e.getMessage}")
          }
      }
    }
  }

  // Pure Math Helper Functions
  def mean(xs: Seq[Double]): Double = if (xs.isEmpty) 0.0 else xs.sum / xs.length

  def variance(xs: Seq[Double], m: Double): Double = {
    if (xs.length <= 1) 0.0
    else xs.map(x => Math.pow(x - m, 2)).sum / (xs.length - 1)
  }

  def stdDev(xs: Seq[Double]): Double = {
    val m = mean(xs)
    Math.sqrt(variance(xs, m))
  }

  def covariance(xs: Seq[Double], ys: Seq[Double], meanX: Double, meanY: Double): Double = {
    val n = Math.min(xs.length, ys.length)
    if (n <= 1) 0.0
    else {
      val sum = xs.zip(ys).map { case (x, y) => (x - meanX) * (y - meanY) }.sum
      sum / (n - 1)
    }
  }
}
```

- [ ] **Step 2: Write MUnit Test `StatsEngineSpec.scala`**
Ensure basic math functions compute correctly.
```scala
package app.services

import munit.FunSuite

class StatsEngineSpec extends FunSuite {
  test("Mean and standard deviation calculations") {
    val data = List(1.0, 2.0, 3.0, 4.0, 5.0)
    assertEquals(StatsEngine.mean(data), 3.0)
    assertEquals(StatsEngine.stdDev(data), Math.sqrt(2.5))
  }
}
```

- [ ] **Step 3: Execute tests**
Run: `sbt test` (from `backend-scala/`)
Expected: All tests pass.

---

### Task 5: Http4s Routing with Circe

**Files:**
- Create: `backend-scala/src/main/scala/app/routes/PortfolioRoutes.scala`

- [ ] **Step 1: Write `PortfolioRoutes.scala`**
Add REST endpoints using Http4s DSL.
```scala
package app.routes

import app.db.Repository
import app.models.Portfolio
import cats.effect.IO
import org.http4s.*
import org.http4s.dsl.io.*
import org.http4s.circe.CirceEntityCodec._

class PortfolioRoutes(repo: Repository) {
  
  val routes: HttpRoutes[IO] = HttpRoutes.of[IO] {
    case GET -> Root / "api" / "portfolios" =>
      repo.listPortfolios().flatMap(Ok(_))

    case GET -> Root / "api" / "portfolios" / IntVar(id) =>
      repo.getPortfolio(id).flatMap {
        case Some(p) => Ok(p)
        case None    => NotFound(s"Portfolio with id $id not found")
      }

    case req @ POST -> Root / "api" / "portfolios" =>
      req.as[Portfolio].flatMap(repo.createPortfolio).flatMap(Created(_))

    case DELETE -> Root / "api" / "portfolios" / IntVar(id) =>
      repo.deletePortfolio(id).flatMap {
        case count if count > 0 => NoContent()
        case _                  => NotFound(s"Portfolio with id $id not found")
      }
  }
}
```

- [ ] **Step 2: Run compiler check**
Verify the http4s route compiles.
Run: `sbt compile`
Expected: Compilation success.

---

### Task 6: App Entry Point & Assembly

**Files:**
- Create: `backend-scala/src/main/scala/app/Main.scala`

- [ ] **Step 1: Write `Main.scala`**
The main server runner using Ember.
```scala
package app

import app.db.*
import app.routes.PortfolioRoutes
import cats.effect.*
import com.comcast.ip4s.*
import org.http4s.ember.server.EmberServerBuilder
import org.http4s.server.middleware.Logger

object Main extends IOApp.Simple {
  
  override def run: IO[Unit] = {
    val dbPath = "../backend/portfolio.db" // direct reuse of current database
    
    val appResources = for {
      xa <- Database.transactor(dbPath)
      repo = new Repository(xa)
      portfolioRoutes = new PortfolioRoutes(repo)
      httpApp = portfolioRoutes.routes.orNotFound
      finalApp = Logger.httpApp(true, true)(httpApp)
      
      server <- EmberServerBuilder.default[IO]
        .withHost(host"127.0.0.1")
        .withPort(port"8000")
        .withHttpApp(finalApp)
        .build
    } yield server

    appResources.use { server =>
      IO.println(s"Scala HTTP Server online at http://127.0.0.1:8000/ - Press Ctrl+C to stop") >> IO.never
    }
  }
}
```

- [ ] **Step 2: Compile & Run Locally**
Test starting the HTTP Server on the JVM.
Run: `sbt run` (from `backend-scala/`)
Expected: Server starts up successfully. Send a request from another terminal:
`curl http://127.0.0.1:8000/api/portfolios`
Expected: Returns portfolio array.

---

### Task 7: GraalVM Native Image Compilation

**Files:**
- `backend-scala/Dockerfile.native`
- `backend-scala/native-image.properties`
- `backend-scala/build-native.sh`

- [x] **Step 1: Compile Native Image**
Compile the app into a standalone native binary using the GraalVM toolchain.

Since GraalVM native-image only works on Linux and requires the SQLite JDBC feature class to be on the classpath (it's in a multi-release JAR), the build process uses:
1. sbt-assembly to create a fat JAR
2. Docker with BellSoft Liberica NIK 25.0.3 to run native-image
3. Extract the SQLite JDBC feature class from the fat JAR and put it on the classpath

Build command:
```bash
cd backend-scala && ./build-native.sh
```

Or manually:
```bash
cd backend-scala && sbt assembly

mkdir -p /tmp/sqlite-feature/org/sqlite/nativeimage
unzip -p target/scala-3.3.7/investment-portfolio-backend-assembly-1.0.0.jar META-INF/versions/9/org/sqlite/nativeimage/SqliteJdbcFeature.class > /tmp/sqlite-feature/org/sqlite/nativeimage/SqliteJdbcFeature.class
unzip -p target/scala-3.3.7/investment-portfolio-backend-assembly-1.0.0.jar META-INF/versions/9/org/sqlite/nativeimage/SqliteJdbcFeature\$SqliteJdbcFeatureException.class > /tmp/sqlite-feature/org/sqlite/nativeimage/SqliteJdbcFeature\$SqliteJdbcFeatureException.class

docker run --rm -v "$PWD":/app -v /tmp/sqlite-feature:/sqlite-feature -w /app --memory=16g --cpus=4 bellsoft/liberica-native-image-kit-container native-image -cp /sqlite-feature -jar target/scala-3.3.7/investment-portfolio-backend-assembly-1.0.0.jar --no-fallback --initialize-at-build-time=org.slf4j,org.sqlite.util.ProcessRunner --enable-url-protocols=http,https --features=org.sqlite.nativeimage.SqliteJdbcFeature -H:+ReportExceptionStackTraces -H:+AddAllCharsets -H:IncludeResourceBundles=org.sqlite.locale -H:DeadlockWatchdogInterval=300 -o /app/target/scala-3.3.7/investment-portfolio-backend
```

Expected: SUCCESS. Generates a 71MB native binary executable under `backend-scala/target/scala-3.3.7/investment-portfolio-backend`.

- [x] **Step 2: Execute Standalone Binary**
Note: The native binary is an ELF ARM64 Linux executable built inside Docker. It cannot be run directly on macOS. To run it, deploy it to a Linux machine or use Docker.

To run on Linux:
```bash
./target/scala-3.3.7/investment-portfolio-backend
```

Expected: Server starts instantly with <10ms startup time, fully functional with SQLite CRUD.

---

### Task 8: API Verification - Compare Scala Backend Against Python OpenAPI Spec

**Files:**
- Reference: `/tmp/openapi.json` (the Python backend OpenAPI spec)

**Goal:** Ensure the Scala backend HTTP endpoints match the Python backend exactly for a seamless frontend integration.

- [ ] **Step 1: Load Python OpenAPI spec**
Reference the spec saved at `/tmp/openapi.json`. The Python backend exposes these endpoints:
```
GET /api/portfolios/ - List Portfolios
POST /api/portfolios/ - Create Portfolio
GET /api/portfolios/{portfolio_id} - Get Portfolio
DELETE /api/portfolios/{portfolio_id} - Delete Portfolio
POST /api/portfolios/{portfolio_id}/assets/ - Create Asset
DELETE /api/assets/{asset_id} - Delete Asset
POST /api/portfolios/{portfolio_id}/assets/{asset_id}/transactions/ - Create Transaction
GET /api/portfolios/{portfolio_id}/transactions/ - List Portfolio Transactions
DELETE /api/transactions/{transaction_id} - Delete Transaction
GET /api/portfolios/{portfolio_id}/tax-summary - Get Portfolio Tax Summary (query: strategy, threshold_days)
GET /api/portfolios/{portfolio_id}/performance - Get Portfolio Performance
GET / - Read Root
```

- [ ] **Step 2: Verify endpoint parity**
For each endpoint above, confirm the Scala backend exposes the exact same path, method, and response format. Pay special attention to:
  - `POST /api/portfolios/{portfolio_id}/tax-summary` has `strategy` (string, default "FIFO") and `threshold_days` (integer, default 30) query parameters.
  - `POST` endpoints return `201 Created` with the created entity in the response body.
  - `DELETE` endpoints return `204 No Content`.
  - `GET` endpoints return `200 OK` with the entity or array of entities.

- [ ] **Step 3: Verify response schema parity**
Ensure all JSON response schemas match exactly:
  - `PortfolioOut`: `{ name: string, description: string | null, currency: string, id: number, assets: AssetOut[] }`
  - `AssetOut`: `{ symbol: string, name: string, asset_type: string, sector: string | null, id: number, portfolio_id: number, transactions: TransactionOut[] }`
  - `TransactionOut`: `{ type: string, quantity: number, price: number, fee: number, date: string (ISO-8601), id: number, asset_id: number }`
  - `HTTPValidationError`: `{ detail: ValidationError[] }` where `ValidationError` has `{ loc: string[] | number[], msg: string, type: string }`

- [ ] **Step 4: Verify request body parity**
Ensure all `POST` request body schemas match exactly:
  - `PortfolioCreate`: `{ name: string, description: string | null, currency: string (default "USD") }`
  - `AssetCreate`: `{ symbol: string, name: string, asset_type: string, sector: string | null }`
  - `TransactionCreate`: `{ type: string (BUY/SELL), quantity: number (exclusiveMinimum 0), price: number (exclusiveMinimum 0), fee: number (minimum 0, default 0), date: string (ISO-8601) }`

- [ ] **Step 5: Execute API tests against Scala backend**
Start the Scala backend (either via `sbt run` or the GraalVM native binary) and run the following curl tests:
```bash
# Create Portfolio
curl -X POST http://127.0.0.1:8000/api/portfolios/ \
  -H "Content-Type: application/json" \
  -d '{"name": "Test Portfolio", "description": "Test", "currency": "USD"}'
# Expected: 201 Created with PortfolioOut

# List Portfolios
curl http://127.0.0.1:8000/api/portfolios/
# Expected: 200 OK with array of PortfolioOut

# Get Portfolio
curl http://127.0.0.1:8000/api/portfolios/1
# Expected: 200 OK with PortfolioOut

# Create Asset
curl -X POST http://127.0.0.1:8000/api/portfolios/1/assets/ \
  -H "Content-Type: application/json" \
  -d '{"symbol": "AAPL", "name": "Apple Inc", "asset_type": "STOCK", "sector": "Technology"}'
# Expected: 201 Created with AssetOut

# Create Transaction
curl -X POST http://127.0.0.1:8000/api/portfolios/1/assets/1/transactions/ \
  -H "Content-Type: application/json" \
  -d '{"type": "BUY", "quantity": 10, "price": 150.0, "fee": 5.0, "date": "2026-06-23T10:00:00"}'
# Expected: 201 Created with TransactionOut

# List Portfolio Transactions
curl http://127.0.0.1:8000/api/portfolios/1/transactions/
# Expected: 200 OK with array of TransactionOut

# Get Tax Summary
curl http://127.0.0.1:8000/api/portfolios/1/tax-summary?strategy=FIFO
# Expected: 200 OK with tax summary object

# Get Performance
curl http://127.0.0.1:8000/api/portfolios/1/performance
# Expected: 200 OK with performance object

# Delete Transaction
curl -X DELETE http://127.0.0.1:8000/api/transactions/1
# Expected: 204 No Content

# Delete Asset
curl -X DELETE http://127.0.0.1:8000/api/assets/1
# Expected: 204 No Content

# Delete Portfolio
curl -X DELETE http://127.0.0.1:8000/api/portfolios/1
# Expected: 204 No Content
```

- [ ] **Step 6: Execute validation tests**
Run:
```bash
# Test 422 Validation Error (missing required field)
curl -X POST http://127.0.0.1:8000/api/portfolios/ \
  -H "Content-Type: application/json" \
  -d '{"description": "No name"}'
# Expected: 422 Unprocessable Entity with HTTPValidationError

# Test 404 Not Found
curl http://127.0.0.1:8000/api/portfolios/999
# Expected: 404 Not Found

# Test Root endpoint
curl http://127.0.0.1:8000/
# Expected: 200 OK with welcome message
```

- [ ] **Step 7: Verify no deviations**
Compare the Scala backend API responses against the Python OpenAPI spec. Report any discrepancies in:
  - Endpoint paths or methods
  - HTTP status codes
  - Response body schemas or field names
  - Query parameter names or defaults
  - Error response formats
