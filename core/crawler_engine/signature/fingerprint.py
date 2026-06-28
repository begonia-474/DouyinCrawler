"""浏览器指纹生成"""

import random


class BrowserFingerprintGenerator:
    """生成浏览器指纹字符串"""

    PLATFORMS = {
        "Edge": "Win32",
        "Chrome": "Win32",
        "Firefox": "Win32",
        "Safari": "MacIntel",
    }

    @staticmethod
    def generate_fingerprint(browser: str = "Edge") -> str:
        """
        生成浏览器指纹

        格式: innerW|innerH|outerW|outerH|screenX|screenY|0|0|sizeW|sizeH|availW|availH|innerW|innerH|24|24|platform
        """
        platform = BrowserFingerprintGenerator.PLATFORMS.get(browser, "Win32")

        inner_width = random.randint(1024, 1920)
        inner_height = random.randint(768, 1080)
        outer_width = inner_width + random.randint(0, 20)
        outer_height = inner_height + random.randint(60, 120)
        screen_x = random.randint(0, 200)
        screen_y = random.randint(0, 200)
        avail_width = random.randint(1920, 2560)
        avail_height = random.randint(1080, 1440)

        return (
            f"{inner_width}|{inner_height}|"
            f"{outer_width}|{outer_height}|"
            f"{screen_x}|{screen_y}|"
            f"0|0|"
            f"{avail_width}|{avail_height}|"
            f"{avail_width}|{avail_height}|"
            f"{inner_width}|{inner_height}|"
            f"24|24|"
            f"{platform}"
        )
