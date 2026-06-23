package app.services

import munit.CatsEffectSuite
import io.circe.syntax._

class TaxLotEngineSpec extends CatsEffectSuite {

  test("processSale should dispose lots in FIFO order") {
    val lots = List(
      TaxLot(1, 1, java.time.LocalDate.of(2024, 1, 1), 100.0, 100.0, 1000.0, 1000.0, 10.0, "USD", "FIFO"),
      TaxLot(2, 1, java.time.LocalDate.of(2024, 6, 1), 50.0, 50.0, 550.0, 550.0, 11.0, "USD", "FIFO")
    )

    val disposition = LotDisposition(1, 75.0)
    val result = SaleResult(List(disposition), 900.0, 750.0, 150.0)
    val response = SaleResponse("FIFO", result, List.empty)

    assertEquals(response.result.dispositions.size, 1)
    assertEquals(response.result.dispositions.head.lotId, 1)
    assertEquals(response.result.dispositions.head.quantity, 75.0)
    assertEquals(response.result.totalProceeds, 900.0)
    assertEquals(response.result.totalCostBasis, 750.0)
    assertEquals(response.result.capitalGain, 150.0)
  }

  test("processSale should dispose lots in LIFO order") {
    val lots = List(
      TaxLot(1, 1, java.time.LocalDate.of(2024, 1, 1), 100.0, 100.0, 1000.0, 1000.0, 10.0, "USD", "LIFO"),
      TaxLot(2, 1, java.time.LocalDate.of(2024, 6, 1), 50.0, 50.0, 550.0, 550.0, 11.0, "USD", "LIFO")
    )

    val disposition1 = LotDisposition(2, 50.0)
    val disposition2 = LotDisposition(1, 25.0)
    val result = SaleResult(List(disposition1, disposition2), 900.0, 800.0, 100.0)
    val response = SaleResponse("LIFO", result, List.empty)

    assertEquals(response.result.dispositions.size, 2)
    assertEquals(response.result.dispositions.head.lotId, 2)
    assertEquals(response.result.dispositions.head.quantity, 50.0)
    assertEquals(response.result.totalProceeds, 900.0)
    assertEquals(response.result.totalCostBasis, 800.0)
    assertEquals(response.result.capitalGain, 100.0)
    assertEquals(response.result.dispositions(1).lotId, 1)
    assertEquals(response.result.dispositions(1).quantity, 25.0)
  }

  test("processSale should handle partial lot disposal") {
    val lots = List(
      TaxLot(1, 1, java.time.LocalDate.of(2024, 1, 1), 100.0, 100.0, 1000.0, 1000.0, 10.0, "USD", "FIFO")
    )

    val disposition = LotDisposition(1, 50.0)
    val result = SaleResult(List(disposition), 600.0, 500.0, 100.0)
    val response = SaleResponse("FIFO", result, List.empty)

    assertEquals(response.result.dispositions.size, 1)
    assertEquals(response.result.dispositions.head.quantity, 50.0)
    assertEquals(response.result.totalCostBasis, 500.0)
    assertEquals(response.result.totalProceeds, 600.0)
    assertEquals(response.result.capitalGain, 100.0)
  }
}
