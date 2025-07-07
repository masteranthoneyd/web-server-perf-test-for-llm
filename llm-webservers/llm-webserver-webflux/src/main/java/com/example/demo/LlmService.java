package com.example.demo;

import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Component;
import reactor.core.publisher.Mono;

import java.time.Duration;
import java.util.concurrent.atomic.AtomicLong;

/**
 * @author yangbingdong1994@gmail.com
 */
@Slf4j
@Component
public class LlmService {

    private static final AtomicLong counter = new AtomicLong();

    public Mono<Void> think() {
        return Mono.delay(Duration.ofSeconds(10))
                .doOnNext(ignored -> log.info("counter:{}", counter.incrementAndGet()))
                .then();
    }
}
