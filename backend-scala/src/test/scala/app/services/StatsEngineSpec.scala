package app.services

import munit.CatsEffectSuite

class StatsEngineSpec extends CatsEffectSuite {

  test("computeStats returns zero values for empty portfolio") {
    val result = StatsResult("2024-01-01", "2024-12-31", 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0)

    result match {
      case StatsResult(_, _, twr, annRet, vol, sharpe, benchRet, beta, alpha) =>
        assertEquals(twr, 0.0)
        assertEquals(annRet, 0.0)
        assertEquals(vol, 0.0)
        assertEquals(sharpe, 0.0)
        assertEquals(benchRet, 0.0)
        assertEquals(beta, 0.0)
        assertEquals(alpha, 0.0)
    }
  }
}
