package app.db

import cats.effect.*
import doobie.*
import doobie.hikari.HikariTransactor
import doobie.util.transactor.Transactor
import org.http4s.ember.client.EmberClientBuilder
import org.http4s.client.Client

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

  val httpClient: Resource[IO, Client[IO]] =
    EmberClientBuilder.default[IO].build

  def repository(xa: HikariTransactor[IO]): Repository[IO] = new Repository(xa)
}
