package com.example.demo;

import org.springframework.boot.autoconfigure.http.client.ClientHttpRequestFactoryBuilderCustomizer;
import org.springframework.boot.http.client.JdkClientHttpRequestFactoryBuilder;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

import java.net.http.HttpClient;
import java.util.concurrent.Executors;
import java.util.function.Consumer;

/**
 * @author bingdong.yang@salesforce-china.com
 */
@Configuration
public class HttpClientConfiguration {

    @Bean
    public CustomClientHttpRequestFactoryBuilderCustomizer customClientHttpRequestFactoryBuilderCustomizer() {
        return new CustomClientHttpRequestFactoryBuilderCustomizer();
    }

    static class CustomClientHttpRequestFactoryBuilderCustomizer implements ClientHttpRequestFactoryBuilderCustomizer<JdkClientHttpRequestFactoryBuilder> {

        @Override
        public JdkClientHttpRequestFactoryBuilder customize(JdkClientHttpRequestFactoryBuilder builder) {
            return builder.withHttpClientCustomizer(builder1 -> {
                builder1.version(HttpClient.Version.HTTP_1_1);
                builder1.executor(Executors.newVirtualThreadPerTaskExecutor());
            });
        }
    }
}
