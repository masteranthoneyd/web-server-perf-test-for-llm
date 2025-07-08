package com.example.demo;

import java.text.SimpleDateFormat;
import java.time.Duration;
import java.util.Calendar;
import java.util.concurrent.ArrayBlockingQueue;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.Future;
import java.util.concurrent.ThreadFactory;
import java.util.concurrent.ThreadPoolExecutor;
import java.util.concurrent.TimeUnit;

/**
 * @author bingdong.yang@salesforce-china.com
 */
public class PinDemo {

    private static volatile Object instance = new Object();
    private static final SimpleDateFormat sdf = new SimpleDateFormat("yyyy-MM-dd HH:mm:ss");

    private static void runTask(int threadNum) {
        realRunTask(threadNum);
    }

    private static void runTaskWithSynchronized(int threadNum) {
        synchronized (instance) {
            realRunTask(threadNum);
        }
    }

    public static String format(Calendar calendar) {
        return sdf.format(calendar.getTime());
    }

    private static void realRunTask(int threadNum) {
        System.out.printf("%s|Test is start ThreadNum is %s %s%n", Thread.currentThread(), threadNum, format(Calendar.getInstance()));
        try {
            Thread.sleep(2000);
        } catch (Exception e) {

        }
        System.out.printf("%s|Test is Over ThreadNum is  %s %s%n", Thread.currentThread(), threadNum, format(Calendar.getInstance()));
    }

    private static ExecutorService getExecutorService(boolean isVirtualThread, boolean useThreadPool) {
        if (useThreadPool) {
            return new ThreadPoolExecutor(50, 50, 1, TimeUnit.MINUTES,
                    new ArrayBlockingQueue<>(100000),
                    isVirtualThread ? Thread.ofVirtual().factory() : Thread.ofPlatform().factory());
        } else {
            ThreadFactory factory = isVirtualThread ?
                    Thread.ofVirtual().name("This-Test-Virtual-Thread-", 0).factory() : Thread.ofPlatform().name(
                    "This-Test-Platform-Thread-", 0).factory();
            return Executors.newThreadPerTaskExecutor(factory);
        }
    }

    /**
     * -Djdk.tracePinnedThreads=full -Djdk.virtualThreadScheduler.parallelism=1 -Djdk.virtualThreadScheduler.maxPoolSize=1 -Djdk.virtualThreadScheduler.minRunnable=1
     * <p>
     * -Djdk.tracePinnedThreads=full -Djdk.virtualThreadScheduler.parallelism=1 -Djdk.virtualThreadScheduler.maxPoolSize=2 -Djdk.virtualThreadScheduler.minRunnable=1
     */
    public static void main(String[] args) throws Exception {
        long start = System.currentTimeMillis();
        ExecutorService executorService = getExecutorService(true, false);
//        Future task1 = executorService.submit(() -> runTask(1));
        Future task1 = executorService.submit(() -> runTaskWithSynchronized(1));
        Future task2 = executorService.submit(() -> runTask(2));
        executorService.close();
        task1.get();
        task2.get();
        System.out.println("End: " + (System.currentTimeMillis() - start) + "ms");
        Thread.sleep(60000L);
    }
}