"""HTTP 請求輔助函數，用於處理 HTTP 錯誤並返回響應對象"""
import requests
from robot.api.deco import keyword


@keyword("GET Request With Error Handling")
def get_request_with_error_handling(url, params=None, headers=None, timeout=10):
    """執行 GET 請求，不拋出 HTTP 錯誤，總是返回響應對象"""
    session = requests.Session()
    response = session.get(url, params=params, headers=headers, timeout=timeout)
    return response


@keyword("POST Request With Error Handling")
def post_request_with_error_handling(url, json_data=None, headers=None, timeout=10):
    """執行 POST 請求，不拋出 HTTP 錯誤，總是返回響應對象"""
    session = requests.Session()
    response = session.post(url, json=json_data, headers=headers, timeout=timeout)
    return response


@keyword("PUT Request With Error Handling")
def put_request_with_error_handling(url, json_data=None, headers=None, timeout=10):
    """執行 PUT 請求，不拋出 HTTP 錯誤，總是返回響應對象"""
    session = requests.Session()
    response = session.put(url, json=json_data, headers=headers, timeout=timeout)
    return response


@keyword("DELETE Request With Error Handling")
def delete_request_with_error_handling(url, headers=None, timeout=10):
    """執行 DELETE 請求，不拋出 HTTP 錯誤，總是返回響應對象"""
    session = requests.Session()
    response = session.delete(url, headers=headers, timeout=timeout)
    return response
