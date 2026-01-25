"""HTTP 請求輔助函數，用於處理 HTTP 錯誤並返回響應對象"""
import requests
import json
from robot.api.deco import keyword

LOG_FILE = '/Users/dennis_leedennis_lee/Documents/GitHub/Smart_Zone/.cursor/debug.log'

def _log(message, data=None):
    """寫入調試日誌"""
    try:
        log_entry = {
            "timestamp": __import__('time').time(),
            "message": message,
            "data": data or {}
        }
        with open(LOG_FILE, 'a') as f:
            f.write(json.dumps(log_entry) + '\n')
    except:
        pass


@keyword("GET Request With Error Handling")
def get_request_with_error_handling(url, params=None, headers=None, timeout=10):
    """執行 GET 請求，不拋出 HTTP 錯誤，總是返回響應對象"""
    _log("GET request", {"url": url, "params": params, "headers": headers})
    session = requests.Session()
    response = session.get(url, params=params, headers=headers, timeout=timeout)
    _log("GET response", {"status": response.status_code, "url": url})
    return response


@keyword("POST Request With Error Handling")
def post_request_with_error_handling(url, json_data=None, headers=None, timeout=10):
    """執行 POST 請求，不拋出 HTTP 錯誤，總是返回響應對象"""
    # 記錄接收到的數據類型和內容
    _log("POST request received", {
        "url": url,
        "json_data_type": str(type(json_data)),
        "json_data": json_data,
        "is_dict": isinstance(json_data, dict),
        "is_str": isinstance(json_data, str),
        "headers": headers
    })
    
    # 處理 Robot Framework 傳遞的數據
    # Robot Framework 可能會將字典轉換為字符串，或者傳遞為空字符串
    if json_data == "${EMPTY}" or json_data == "":
        json_data = {}
    elif json_data is None:
        json_data = {}
    elif isinstance(json_data, str):
        # 如果是字符串，嘗試解析為 JSON
        if json_data.strip().startswith("{"):
            try:
                json_data = json.loads(json_data)
            except:
                json_data = {}
        else:
            # 空字符串或非 JSON 字符串
            json_data = {}
    # 如果已經是字典，直接使用
    
    _log("POST request processed", {"url": url, "json_data": json_data, "headers": headers})
    session = requests.Session()
    response = session.post(url, json=json_data, headers=headers, timeout=timeout)
    _log("POST response", {"status": response.status_code, "url": url, "response_body": response.text[:200]})
    return response


@keyword("PUT Request With Error Handling")
def put_request_with_error_handling(url, json_data=None, headers=None, timeout=10):
    """執行 PUT 請求，不拋出 HTTP 錯誤，總是返回響應對象"""
    # 記錄接收到的數據類型和內容
    _log("PUT request received", {
        "url": url,
        "json_data_type": str(type(json_data)),
        "json_data": json_data,
        "is_dict": isinstance(json_data, dict),
        "is_str": isinstance(json_data, str),
        "headers": headers
    })
    
    # 處理 Robot Framework 傳遞的數據
    # Robot Framework 可能會將字典轉換為字符串，或者傳遞為空字符串
    if json_data == "${EMPTY}" or json_data == "":
        json_data = {}
    elif json_data is None:
        json_data = {}
    elif isinstance(json_data, str):
        # 如果是字符串，嘗試解析為 JSON
        if json_data.strip().startswith("{"):
            try:
                json_data = json.loads(json_data)
            except:
                json_data = {}
        else:
            # 空字符串或非 JSON 字符串
            json_data = {}
    # 如果已經是字典，直接使用
    
    _log("PUT request processed", {"url": url, "json_data": json_data, "headers": headers})
    session = requests.Session()
    response = session.put(url, json=json_data, headers=headers, timeout=timeout)
    _log("PUT response", {"status": response.status_code, "url": url, "response_body": response.text[:200]})
    return response


@keyword("DELETE Request With Error Handling")
def delete_request_with_error_handling(url, headers=None, timeout=10):
    """執行 DELETE 請求，不拋出 HTTP 錯誤，總是返回響應對象"""
    _log("DELETE request", {"url": url, "headers": headers})
    session = requests.Session()
    response = session.delete(url, headers=headers, timeout=timeout)
    _log("DELETE response", {"status": response.status_code, "url": url})
    return response
