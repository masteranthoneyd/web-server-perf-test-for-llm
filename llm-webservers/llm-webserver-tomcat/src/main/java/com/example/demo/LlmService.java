package com.example.demo;

import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Component;

import java.time.Duration;
import java.time.temporal.ChronoUnit;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicLong;

/**
 * @author yangbingdong1994@gmail.com
 */
@Slf4j
@Component
public class LlmService {

    private static final AtomicLong counter = new AtomicLong();

    public void think() {
        try {
            Thread.sleep(Duration.of(10, ChronoUnit.SECONDS));
            log.info("counter:{}", counter.incrementAndGet());
        } catch (InterruptedException e) {
            throw new RuntimeException(e);
        }
    }
}
