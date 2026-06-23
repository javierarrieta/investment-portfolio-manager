name := "investment-portfolio-backend"
version := "1.0.0"
scalaVersion := "3.3.7"

val CatsEffectVersion = "3.5.4"
val Http4sVersion     = "0.23.25"
val CirceVersion      = "0.14.9"
val DoobieVersion     = "1.0.0-RC5"
val LogbackVersion    = "1.5.6"
val MUnitVersion      = "1.0.0"
val MUnitCEVersion    = "1.0.7"

libraryDependencies ++= Seq(
  "org.typelevel"         %% "cats-effect"         % CatsEffectVersion,
  "com.comcast"           %% "ip4s-core"           % "3.4.0",
  "org.http4s"            %% "http4s-ember-server" % Http4sVersion,
  "org.http4s"            %% "http4s-ember-client" % Http4sVersion,
  "org.http4s"            %% "http4s-client"       % Http4sVersion,
  "org.http4s"            %% "http4s-dsl"          % Http4sVersion,
  "org.http4s"            %% "http4s-circe"        % Http4sVersion,
  "org.http4s"            %% "http4s-laws"         % Http4sVersion % Test,
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

mainClass in Compile := Some("app.Main")

assemblyMergeStrategy in assembly := {
  case "module-info.class" =>
    MergeStrategy.first
  case "pom.xml" =>
    MergeStrategy.discard
  case "pom.properties" =>
    MergeStrategy.discard
  case PathList("META-INF", "versions", "9", "module-info.class") =>
    MergeStrategy.discard
  case PathList("META-INF", "module-info.class") =>
    MergeStrategy.discard
  case x =>
    val oldStrategy = (assemblyMergeStrategy in assembly).value
    oldStrategy(x)
}


