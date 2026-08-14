from httpcli.fetch import fetch
from httpcli.display import (
    d01, d02, d03, d04, d05, d06, d07, d08, d09, d10,
    d11, d12, d13, d14, d15, d16, d17, d18, d19, d20,
    d21, d22, d23, d24, d25, d26, d27, d28, d29, d30, d31,
)


def main(url: str = "https://example.com") -> str:
    body = fetch(url)
    d01(body); d02(body); d03(body); d04(body); d05(body); d06(body); d07(body)
    d08(body); d09(body); d10(body); d11(body); d12(body); d13(body); d14(body)
    d15(body); d16(body); d17(body); d18(body); d19(body); d20(body); d21(body)
    d22(body); d23(body); d24(body); d25(body); d26(body); d27(body); d28(body)
    d29(body); d30(body); d31(body)
    return body


if __name__ == "__main__":
    print(main())
