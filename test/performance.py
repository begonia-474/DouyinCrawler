"""
性能测试脚本
测试 PyO3 调用 vs HTTP 代理的速度
"""

import time
import asyncio
import statistics
from typing import Callable, Any


def benchmark(func: Callable, iterations: int = 10, warmup: int = 3) -> dict:
    """
    性能测试函数
    
    Args:
        func: 要测试的函数
        iterations: 测试次数
        warmup: 预热次数
    
    Returns:
        测试结果字典
    """
    # 预热
    for _ in range(warmup):
        func()
    
    # 测试
    times = []
    for _ in range(iterations):
        start = time.perf_counter()
        result = func()
        end = time.perf_counter()
        times.append(end - start)
    
    return {
        "min": min(times),
        "max": max(times),
        "mean": statistics.mean(times),
        "median": statistics.median(times),
        "stdev": statistics.stdev(times) if len(times) > 1 else 0,
        "iterations": iterations,
        "result": result,
    }


async def async_benchmark(func: Callable, iterations: int = 10, warmup: int = 3) -> dict:
    """
    异步性能测试函数
    """
    # 预热
    for _ in range(warmup):
        await func()
    
    # 测试
    times = []
    for _ in range(iterations):
        start = time.perf_counter()
        result = await func()
        end = time.perf_counter()
        times.append(end - start)
    
    return {
        "min": min(times),
        "max": max(times),
        "mean": statistics.mean(times),
        "median": statistics.median(times),
        "stdev": statistics.stdev(times) if len(times) > 1 else 0,
        "iterations": iterations,
        "result": result,
    }


def print_result(name: str, result: dict):
    """打印测试结果"""
    print(f"\n{'='*50}")
    print(f"测试: {name}")
    print(f"{'='*50}")
    print(f"迭代次数: {result['iterations']}")
    print(f"最小时间: {result['min']*1000:.2f} ms")
    print(f"最大时间: {result['max']*1000:.2f} ms")
    print(f"平均时间: {result['mean']*1000:.2f} ms")
    print(f"中位时间: {result['median']*1000:.2f} ms")
    print(f"标准差:   {result['stdev']*1000:.2f} ms")


if __name__ == "__main__":
    print("性能测试脚本")
    print("请在 Tauri 应用中运行测试")
