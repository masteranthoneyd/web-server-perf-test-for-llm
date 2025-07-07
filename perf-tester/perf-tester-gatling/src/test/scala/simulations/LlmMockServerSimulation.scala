package simulations

import io.gatling.core.Predef._
import io.gatling.core.structure.ScenarioBuilder
import io.gatling.http.Predef._
import io.gatling.http.protocol.HttpProtocolBuilder

import scala.concurrent.duration._

class LlmMockServerSimulation extends Simulation {

  val httpProtocol: HttpProtocolBuilder = http
    .baseUrl("http://127.0.0.1:9091/")
    .acceptHeader("application/json")
    .contentTypeHeader("application/json")
    .userAgentHeader("Gatling Performance Test")

  val requestBody = """
  {
    "model": "gpt-3.5-turbo",
    "messages": [
      {
        "role": "user",
        "content": "Hello, how are you?"
      }
    ]
  }
  """

  val scn: ScenarioBuilder = scenario("LLM API Chat Completions Test")
    .forever(
      exec(
        http("Chat Completions Request")
          .post("/compatible-mode/v1/chat/completions")
          .body(StringBody(requestBody))
          .check(status.is(200))
          .check(jsonPath("$.choices[0].message.content").exists)
      )
    )

  setUp(
    scn.inject(rampUsers(5000).during(10.seconds))
       .protocols(httpProtocol)
  )
  .maxDuration(120.seconds)
} 