"""A-Bogus 签名算法"""

import time
import random
from gmssl import sm3, func


class StringProcessor:
    """字符串处理工具"""

    @staticmethod
    def to_char_array(s: str) -> list[int]:
        return [ord(c) for c in s]

    @staticmethod
    def to_char_str(arr: list[int]) -> str:
        return "".join([chr(i) for i in arr])

    @staticmethod
    def to_ord_str(arr: list[int]) -> str:
        return "".join([chr(i) for i in arr])

    @staticmethod
    def js_shift_right(val: int, n: int) -> int:
        return (val % 0x100000000) >> n

    @staticmethod
    def generate_random_bytes(length: int = 3) -> str:
        def gen_one() -> list[str]:
            _rd = int(random.random() * 10000)
            return [
                chr(((_rd & 255) & 170) | 1),
                chr(((_rd & 255) & 85) | 2),
                chr((StringProcessor.js_shift_right(_rd, 8) & 170) | 5),
                chr((StringProcessor.js_shift_right(_rd, 8) & 85) | 40),
            ]

        result = []
        for _ in range(length):
            result.extend(gen_one())
        return "".join(result)


class CryptoUtility:
    """加密工具"""

    SALT = "cus"

    ALPHABET_0 = "Dkdpgh2ZmsQB80/MfvV36XI1R45-WUAlEixNLwoqYTOPuzKFjJnry79HbGcaStCe"
    ALPHABET_1 = "ckdp1h4ZKsUB80/Mfvw36XIgR25+WQAlEi7NLboqYTOPuzmFjJnryx9HVGDaStCe"
    ALPHABETS = [ALPHABET_0, ALPHABET_1]

    # fmt: off
    BIG_ARRAY = [
        121, 243,  55, 234, 103,  36,  47, 228,  30, 231, 106,   6, 115,  95,  78, 101, 250, 207, 198,  50,
        139, 227, 220, 105,  97, 143,  34,  28, 194, 215,  18, 100, 159, 160,  43,   8, 169, 217, 180, 120,
        247,  45,  90,  11,  27, 197,  46,   3,  84,  72,   5,  68,  62,  56, 221,  75, 144,  79,  73, 161,
        178,  81,  64, 187, 134, 117, 186, 118,  16, 241, 130,  71,  89, 147, 122, 129,  65,  40,  88, 150,
        110, 219, 199, 255, 181, 254,  48,   4, 195, 248, 208,  32, 116, 167,  69, 201,  17, 124, 125, 104,
         96,  83,  80, 127, 236, 108, 154, 126, 204,  15,  20, 135, 112, 158,  13,   1, 188, 164, 210, 237,
        222,  98, 212,  77, 253,  42, 170, 202,  26,  22,  29, 182, 251,  10, 173, 152,  58, 138,  54, 141,
        185,  33, 157,  31, 252, 132, 233, 235, 102, 196, 191, 223, 240, 148,  39, 123,  92,  82, 128, 109,
         57,  24,  38, 113, 209, 245,   2, 119, 153, 229, 189, 214, 230, 174, 232,  63,  52, 205,  86, 140,
         66, 175, 111, 171, 246, 133, 238, 193,  99,  60,  74,  91, 225,  51,  76,  37, 145, 211, 166, 151,
        213, 206,   0, 200, 244, 176, 218,  44, 184, 172,  49, 216,  93, 168,  53,  21, 183,  41,  67,  85,
        224, 155, 226, 242,  87, 177, 146,  70, 190,  12, 162,  19, 137, 114,  25, 165, 163, 192,  23,  59,
          9,  94, 179, 107,  35,   7, 142, 131, 239, 203, 149, 136,  61, 249,  14, 156
    ]
    # fmt: on

    def __init__(self):
        self._big_array = list(self.BIG_ARRAY)

    @staticmethod
    def sm3_to_array(data: str | list[int]) -> list[int]:
        if isinstance(data, str):
            data_bytes = data.encode("utf-8")
        else:
            data_bytes = bytes(data)
        hex_result = sm3.sm3_hash(func.bytes_to_list(data_bytes))
        return [int(hex_result[i:i + 2], 16) for i in range(0, len(hex_result), 2)]

    @staticmethod
    def rc4_encrypt(key: bytes, plaintext: str) -> bytes:
        S = list(range(256))
        j = 0
        for i in range(256):
            j = (j + S[i] + key[i % len(key)]) % 256
            S[i], S[j] = S[j], S[i]
        i = j = 0
        ciphertext = []
        for ch in plaintext:
            i = (i + 1) % 256
            j = (j + S[i]) % 256
            S[i], S[j] = S[j], S[i]
            ciphertext.append(ord(ch) ^ S[(S[i] + S[j]) % 256])
        return bytes(ciphertext)

    def double_sm3(self, data: str) -> list[int]:
        """SM3(SM3(SALT + data))"""
        first = self.sm3_to_array(self.SALT + data)
        return self.sm3_to_array(first)

    def transform_bytes(self, bytes_list: list[int]) -> str:
        """流密码加密 — 使用 256 元素 big_array 进行 XOR + 交换"""
        self._big_array = list(self.BIG_ARRAY)
        bytes_str = StringProcessor.to_char_str(bytes_list)
        result = []
        index_b = self._big_array[1]
        initial_value = 0

        for idx, ch in enumerate(bytes_str):
            if idx == 0:
                initial_value = self._big_array[index_b]
                sum_init = index_b + initial_value
                self._big_array[1] = initial_value
                self._big_array[index_b] = index_b
            else:
                sum_init = initial_value + value_e

            ch_val = ord(ch)
            sum_init %= len(self._big_array)
            value_f = self._big_array[sum_init]
            result.append(chr(ch_val ^ value_f))

            value_e = self._big_array[(idx + 2) % len(self._big_array)]
            sum_init = (index_b + value_e) % len(self._big_array)
            initial_value = self._big_array[sum_init]
            self._big_array[sum_init] = self._big_array[(idx + 2) % len(self._big_array)]
            self._big_array[(idx + 2) % len(self._big_array)] = initial_value
            index_b = sum_init

        return "".join(result)

    @staticmethod
    def abogus_encode(data: str, alphabet_index: int) -> str:
        """自定义 Base64 编码"""
        alphabet = CryptoUtility.ALPHABETS[alphabet_index]
        result = []
        for i in range(0, len(data), 3):
            if i + 2 < len(data):
                n = (ord(data[i]) << 16) | (ord(data[i + 1]) << 8) | ord(data[i + 2])
            elif i + 1 < len(data):
                n = (ord(data[i]) << 16) | (ord(data[i + 1]) << 8)
            else:
                n = ord(data[i]) << 16

            for j, k in zip(range(18, -1, -6), (0xFC0000, 0x03F000, 0x0FC0, 0x3F)):
                if j == 6 and i + 1 >= len(data):
                    break
                if j == 0 and i + 2 >= len(data):
                    break
                result.append(alphabet[(n & k) >> j])

        result.append("=" * ((4 - len(result) % 4) % 4))
        return "".join(result)


class ABogus:
    """A-Bogus 签名生成器"""

    UA_KEY = b"\x00\x01\x0E"

    OPTIONS = [0, 1, 14]

    # fmt: off
    SORT_INDEX = [
        18, 20, 52, 26, 30, 34, 58, 38, 40, 53, 42, 21, 27, 54, 55, 31, 35, 57, 39, 41, 43, 22, 28,
        32, 60, 36, 23, 29, 33, 37, 44, 45, 59, 46, 47, 48, 49, 50, 24, 25, 65, 66, 70, 71
    ]
    SORT_INDEX_2 = [
        18, 20, 26, 30, 34, 38, 40, 42, 21, 27, 31, 35, 39, 41, 43, 22, 28, 32, 36, 23, 29, 33, 37,
        44, 45, 46, 47, 48, 49, 50, 24, 25, 52, 53, 54, 55, 57, 58, 59, 60, 65, 66, 70, 71
    ]
    # fmt: on

    def __init__(self, user_agent: str, fingerprint: str):
        self.ua = user_agent
        self.fp = fingerprint
        self.aid = 6383
        self.page_id = 0
        self.crypto = CryptoUtility()

    def generate(self, params: str, body: str = "") -> tuple[str, str, str, str]:
        """
        生成 A-Bogus 签名

        Args:
            params: URL 查询参数字符串
            body: POST 请求体（GET 请求为空字符串）

        Returns:
            (带签名的参数字符串, a_bogus 值, User-Agent, body)
        """
        # 开始加密时间
        start_ms = int(time.time() * 1000)

        # 双重 SM3 哈希
        array1 = self.crypto.double_sm3(params)
        array2 = self.crypto.double_sm3(body)
        ua_encrypted = CryptoUtility.rc4_encrypt(self.UA_KEY, self.ua)
        ua_b64 = CryptoUtility.abogus_encode(StringProcessor.to_ord_str(ua_encrypted), 1)
        array3 = self.crypto.sm3_to_array(ua_b64)

        # 结束加密时间
        end_ms = int(time.time() * 1000)

        # 构建 ab_dir
        ab_dir = {
            8: 3,
            15: {
                "aid": self.aid, "pageId": self.page_id,
                "boe": False, "ddrt": 8.5,
                "paths": [
                    "^/webcast/", "^/aweme/v1/", "^/aweme/v2/",
                    "/v1/message/send", "^/live/", "^/captcha/", "^/ecom/",
                ],
                "track": {"mode": 0, "delay": 300, "paths": []},
                "dump": True, "rpU": "",
            },
            18: 44,
            19: [1, 0, 1, 0, 1],
            66: 0, 69: 0, 70: 0, 71: 0,
        }

        # 开始时间字节
        ab_dir[20] = (start_ms >> 24) & 255
        ab_dir[21] = (start_ms >> 16) & 255
        ab_dir[22] = (start_ms >> 8) & 255
        ab_dir[23] = start_ms & 255
        ab_dir[24] = int(start_ms / 256**4) & 255
        ab_dir[25] = int(start_ms / 256**5) & 255

        # 请求选项
        ab_dir[26] = (self.OPTIONS[0] >> 24) & 255
        ab_dir[27] = (self.OPTIONS[0] >> 16) & 255
        ab_dir[28] = (self.OPTIONS[0] >> 8) & 255
        ab_dir[29] = self.OPTIONS[0] & 255
        ab_dir[30] = int(self.OPTIONS[1] / 256) & 255
        ab_dir[31] = (self.OPTIONS[1] % 256) & 255
        ab_dir[32] = (self.OPTIONS[1] >> 24) & 255
        ab_dir[33] = (self.OPTIONS[1] >> 16) & 255
        ab_dir[34] = (self.OPTIONS[2] >> 24) & 255
        ab_dir[35] = (self.OPTIONS[2] >> 16) & 255
        ab_dir[36] = (self.OPTIONS[2] >> 8) & 255
        ab_dir[37] = self.OPTIONS[2] & 255

        # hash 选择字节
        ab_dir[38] = array1[21]
        ab_dir[39] = array1[22]
        ab_dir[40] = array2[21]
        ab_dir[41] = array2[22]
        ab_dir[42] = array3[23]
        ab_dir[43] = array3[24]

        # 结束时间字节
        ab_dir[44] = (end_ms >> 24) & 255
        ab_dir[45] = (end_ms >> 16) & 255
        ab_dir[46] = (end_ms >> 8) & 255
        ab_dir[47] = end_ms & 255
        ab_dir[48] = ab_dir[8]
        ab_dir[49] = int(end_ms / 256**4) & 255
        ab_dir[50] = int(end_ms / 256**5) & 255

        # pageId / aid 字节
        ab_dir[51] = (self.page_id >> 24) & 255
        ab_dir[52] = (self.page_id >> 16) & 255
        ab_dir[53] = (self.page_id >> 8) & 255
        ab_dir[54] = self.page_id & 255
        ab_dir[55] = self.page_id
        ab_dir[56] = self.aid
        ab_dir[57] = self.aid & 255
        ab_dir[58] = (self.aid >> 8) & 255
        ab_dir[59] = (self.aid >> 16) & 255
        ab_dir[60] = (self.aid >> 24) & 255

        # 浏览器指纹长度
        ab_dir[64] = len(self.fp)
        ab_dir[65] = len(self.fp)

        # 排序提取
        sorted_values = [ab_dir.get(i, 0) for i in self.SORT_INDEX]

        # 浏览器指纹字节
        fp_array = StringProcessor.to_char_array(self.fp)

        # XOR 校验
        ab_xor = 0
        for i in range(len(self.SORT_INDEX_2) - 1):
            if i == 0:
                ab_xor = ab_dir.get(self.SORT_INDEX_2[i], 0)
            ab_xor ^= ab_dir.get(self.SORT_INDEX_2[i + 1], 0)

        sorted_values.extend(fp_array)
        sorted_values.append(ab_xor)

        # 流密码加密 + Base64 编码
        abogus_bytes = (
            StringProcessor.generate_random_bytes()
            + self.crypto.transform_bytes(sorted_values)
        )
        abogus = CryptoUtility.abogus_encode(abogus_bytes, 0)

        signed_params = f"{params}&a_bogus={abogus}"
        return signed_params, abogus, self.ua, body


class ABogusManager:
    """A-Bogus 签名管理器"""

    @staticmethod
    def model_2_endpoint(user_agent: str, base_endpoint: str, params: dict, body: str = "") -> str:
        """从参数模型生成带签名的完整 URL"""
        from core.signature.fingerprint import BrowserFingerprintGenerator
        fp = BrowserFingerprintGenerator.generate_fingerprint("Edge")

        param_str = "&".join(f"{k}={v}" for k, v in params.items())
        ab = ABogus(user_agent, fp)
        signed, _, _, _ = ab.generate(param_str, body)

        sep = "&" if "?" in base_endpoint else "?"
        return f"{base_endpoint}{sep}{signed}"


if __name__ == "__main__":
    ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36 Edg/131.0.0.0"
    from core.signature.fingerprint import BrowserFingerprintGenerator
    fp = BrowserFingerprintGenerator.generate_fingerprint("Edge")
    ab = ABogus(user_agent=ua, fingerprint=fp)

    params = "device_platform=webapp&aid=6383&channel=channel_pc_web&version_code=290100&version_name=29.1.0&cookie_enabled=true&screen_width=1920&screen_height=1080&browser_language=zh-CN&browser_platform=Win32&browser_name=Edge&browser_version=131.0.0.0&browser_online=true&engine_name=Blink&engine_version=131.0.0.0&os_name=Windows&os_version=10&cpu_core_num=12&device_memory=8&platform=PC&downlink=10&effective_type=4g&round_trip_time=50"

    result = ab.generate(params)
    print(f"a_bogus: {result[1]}")
    print(f"Signed: {result[0][:100]}...")
