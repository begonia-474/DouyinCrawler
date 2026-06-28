"""X-Bogus 签名算法"""

import time
import base64
import hashlib


class XBogus:
    """X-Bogus 签名生成器"""

    # 自定义 Base64 字母表
    ENCODE_TABLE = "Dkdpgh4ZKsQB80/Mfvw36XI1R25-WUAlEi7NLboqYTOPuzmFjJnryx9HVGcaStCe="

    # Hex 字符到索引的映射
    HEX_MAP = [
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, 10, 11, 12, 13, 14, 15
    ]

    UA_KEY = b"\x00\x01\x0c"

    def __init__(self, user_agent: str):
        self.ua = user_agent

    @staticmethod
    def _rc4(key: bytes, data: bytes) -> bytearray:
        """RC4 加密"""
        S = list(range(256))
        j = 0
        for i in range(256):
            j = (j + S[i] + key[i % len(key)]) % 256
            S[i], S[j] = S[j], S[i]

        result = bytearray()
        i = j = 0
        for byte in data:
            i = (i + 1) % 256
            j = (j + S[i]) % 256
            S[i], S[j] = S[j], S[i]
            result.append(byte ^ S[(S[i] + S[j]) % 256])
        return result

    @staticmethod
    def _md5(data: bytes) -> str:
        """计算 MD5"""
        return hashlib.md5(data).hexdigest()

    @classmethod
    def _hex_to_bytes(cls, hex_str: str) -> list[int]:
        """将 MD5 hex 字符串转为字节数组"""
        if len(hex_str) > 32:
            return [ord(c) for c in hex_str]
        result = []
        for i in range(0, len(hex_str), 2):
            result.append((cls.HEX_MAP[ord(hex_str[i])] << 4) | cls.HEX_MAP[ord(hex_str[i + 1])])
        return result

    @classmethod
    def _double_md5(cls, data: bytes) -> list[int]:
        """双重 MD5"""
        first = cls._md5(data)
        second = cls._md5(bytes(cls._hex_to_bytes(first)))
        return cls._hex_to_bytes(second)

    @classmethod
    def _encode_chunk(cls, a: int, b: int, c: int) -> str:
        """3字节编码为4字符"""
        x = ((a & 255) << 16) | ((b & 255) << 8) | c
        return (
            cls.ENCODE_TABLE[(x & 16515072) >> 18]
            + cls.ENCODE_TABLE[(x & 258048) >> 12]
            + cls.ENCODE_TABLE[(x & 4032) >> 6]
            + cls.ENCODE_TABLE[x & 63]
        )

    def generate(self, url_params: str) -> tuple[str, str, str]:
        """
        生成 X-Bogus 签名

        Args:
            url_params: URL 查询参数字符串

        Returns:
            (带签名的参数字符串, X-Bogus 值, User-Agent)
        """
        # 1. RC4加密 UA → Base64 → MD5 → 字节数组
        ua_encrypted = self._rc4(self.UA_KEY, self.ua.encode("ISO-8859-1"))
        ua_b64 = base64.b64encode(ua_encrypted).decode("ISO-8859-1")
        array1 = self._hex_to_bytes(self._md5(ua_b64.encode()))

        # 2. 空字符串MD5常量 → 再MD5 → 字节数组
        empty_md5 = "d41d8cd98f00b204e9800998ecf8427e"
        array2 = self._hex_to_bytes(self._md5(bytes(self._hex_to_bytes(empty_md5))))

        # 3. URL参数 → 双重MD5
        url_params_array = self._double_md5(url_params.encode())

        # 4. 构建20字节数组
        timer = int(time.time())
        ct = 536919696

        new_array = [
            64, 0.00390625, 1, 12,
            url_params_array[14], url_params_array[15],
            array2[14], array2[15],
            array1[14], array1[15],
            (timer >> 24) & 255, (timer >> 16) & 255, (timer >> 8) & 255, timer & 255,
            (ct >> 24) & 255, (ct >> 16) & 255, (ct >> 8) & 255, ct & 255,
        ]

        # XOR 校验
        xor_result = new_array[0]
        for i in range(1, len(new_array)):
            b = new_array[i]
            if isinstance(b, float):
                b = int(b)
            xor_result ^= b
        new_array.append(xor_result)

        # 5. 解交织（确保所有值为整数）
        int_array = [int(v) for v in new_array]
        odd = []
        even = []
        for i in range(0, len(int_array), 2):
            odd.append(int_array[i])
            if i + 1 < len(int_array):
                even.append(int_array[i + 1])
        merged = odd + even

        # 6. 编码转换 + RC4加密
        # f2 encoding_conversion(a, b, c, e, d, t, f, r, n, o, i, _, x, u, s, l, v, h, p)
        #   -> [a, int(i), b, _, c, x, e, u, d, s, t, l, f, v, r, h, n, p, o]
        encoded = bytes([
            merged[0], int(merged[10]),
            merged[1], merged[11], merged[2], merged[12],
            merged[3], merged[13], merged[4], merged[14],
            merged[5], merged[15], merged[6], merged[16],
            merged[7], merged[17], merged[8], merged[18], merged[9],
        ]).decode("ISO-8859-1")

        rc4_result = self._rc4(
            "ÿ".encode("ISO-8859-1"),
            encoded.encode("ISO-8859-1"),
        )

        garbled = chr(2) + chr(255) + rc4_result.decode("ISO-8859-1")

        # 7. 自定义 Base64 编码
        xb = ""
        for i in range(0, len(garbled), 3):
            xb += self._encode_chunk(
                ord(garbled[i]),
                ord(garbled[i + 1]),
                ord(garbled[i + 2]),
            )

        signed_params = f"{url_params}&X-Bogus={xb}"
        return signed_params, xb, self.ua


if __name__ == "__main__":
    ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36 Edg/122.0.0.0"
    xb = XBogus(ua)
    params = "device_platform=webapp&aid=6383&channel=channel_pc_web&sec_user_id=MS4wLjABAAAA"
    result = xb.generate(params)
    print(f"X-Bogus: {result[1]}")
    print(f"Signed: {result[0]}")
