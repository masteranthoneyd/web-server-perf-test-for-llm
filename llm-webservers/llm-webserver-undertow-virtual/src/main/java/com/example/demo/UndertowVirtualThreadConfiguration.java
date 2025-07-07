package com.example.demo;

import org.springframework.boot.web.embedded.undertow.UndertowDeploymentInfoCustomizer;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

import java.util.concurrent.Executors;

/**
 * @author bingdong.yang@salesforce-china.com
 */
@Configuration(proxyBeanMethods = false)
public class UndertowVirtualThreadConfiguration {

    @Bean
    public UndertowDeploymentInfoCustomizer undertowDeploymentInfoCustomizer() {
        return deploymentInfo -> deploymentInfo.setExecutor(Executors.newVirtualThreadPerTaskExecutor());
    }
}
