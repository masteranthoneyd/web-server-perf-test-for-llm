package com.example.demo;

import lombok.RequiredArgsConstructor;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RestController;

/**
 * @author bingdong.yang@salesforce-china.com
 */
@RequiredArgsConstructor
@RestController
public class TestController {

    private final HttpClientTestComponent httpClientTestComponent;
    @GetMapping("/test")
    public void test() {
        httpClientTestComponent.test();
    }
}
