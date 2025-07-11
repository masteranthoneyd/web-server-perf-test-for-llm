package com.example.demo;

import lombok.extern.slf4j.Slf4j;
import org.springframework.ai.openai.api.OpenAiApi;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.http.ResponseEntity;
import org.springframework.stereotype.Component;
import org.springframework.web.client.RestClient;

import java.util.concurrent.atomic.AtomicLong;

/**
 * @author yangbingdong1994@gmail.com
 */
@Slf4j
@Component
public class LlmService {

    private static final AtomicLong counter = new AtomicLong();

    @Autowired
    private RestClient.Builder builder;

    public void callLlm() {
        ResponseEntity<OpenAiApi.ChatCompletion> entity = builder.build().post()
                .uri("http://10.5.148.136:8080/compatible-mode/v1/chat/completions")
                .retrieve()
                .toEntity(OpenAiApi.ChatCompletion.class);
//        log.info("result: {}", entity.getBody());
//        log.info("counter:{}", counter.incrementAndGet());
    }
}
