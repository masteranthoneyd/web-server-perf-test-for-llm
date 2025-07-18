package com.example.demo;

import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;

@SpringBootApplication
public class LlmWebServerUndertowFineTuneApplication {

    /**
     * -Djdk.virtualThreadScheduler.maxPoolSize=2 -Xmx2G -Xms1G -XX:+UseZGC
     */
    public static void main(String[] args) {
        SpringApplication.run(LlmWebServerUndertowFineTuneApplication.class, args);
    }

}
