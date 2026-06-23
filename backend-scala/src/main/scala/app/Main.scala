package app

import cats.effect._
import cats.effect.std.Console
import cats.implicits._
import org.http4s.ember.server.EmberServerBuilder
import org.http4s.server.middleware.Logger
import app.db.Database
import app.routes.PortfolioRoutes
import doobie.hikari.HikariTransactor
import com.comcast.ip4s.Host
import com.comcast.ip4s.Port

object Main {

  def main(args: Array[String]): Unit = {
    import cats.effect.unsafe.implicits.global
    run(args.toList).unsafeRunSync()
  }

  def run(args: List[String]): IO[ExitCode] =
    for {
      _ <- Console[IO].println("Starting Investment Portfolio Manager...")
      _ <- Database.transactor("backend/portfolio.db").use { xa =>
        val repo = new app.db.Repository[IO](xa)
        val routes = PortfolioRoutes.routes(repo, xa)
        EmberServerBuilder
          .default[IO]
          .withHost(Host.fromString("127.0.0.1").getOrElse(Host.fromString("localhost").get))
          .withPort(Port.fromInt(8000).getOrElse(Port.fromInt(8000).get))
          .withHttpApp(Logger.httpApp[IO](true, true)(routes.orNotFound))
          .build
          .onFinalize(Console[IO].println("Server stopped").void)
          .use { _ =>
            Console[IO].println("Server started on http://127.0.0.1:8000/api").as(ExitCode.Success)
          }
      }
    } yield ExitCode.Success
}
