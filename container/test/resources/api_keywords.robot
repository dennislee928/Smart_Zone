*** Settings ***
Documentation    可重用的 API 請求關鍵字庫
Library           RequestsLibrary
Library           Collections
Library           ${EXECDIR}/resources/http_helpers.py
Variables         ${EXECDIR}/variables/config.py

*** Keywords ***
GET Request
    [Documentation]    執行 GET 請求（會捕獲 HTTP 錯誤並返回響應）
    [Arguments]    ${url}    ${params}=${EMPTY}    ${headers}=${EMPTY}
    ${has_params}=    Run Keyword And Return Status    Evaluate    '${params}' != '${EMPTY}' and isinstance(${params}, dict) if '${params}' != '${EMPTY}' else False
    ${has_headers}=    Run Keyword And Return Status    Evaluate    '${headers}' != '${EMPTY}' and isinstance(${headers}, dict) if '${headers}' != '${EMPTY}' else False
    ${response}=    Run Keyword If    ${has_params} and ${has_headers}    GET Request With Error Handling    ${url}    params=${params}    headers=${headers}    timeout=${API_TIMEOUT}
    ...    ELSE IF    ${has_params}    GET Request With Error Handling    ${url}    params=${params}    timeout=${API_TIMEOUT}
    ...    ELSE IF    ${has_headers}    GET Request With Error Handling    ${url}    headers=${headers}    timeout=${API_TIMEOUT}
    ...    ELSE    GET Request With Error Handling    ${url}    timeout=${API_TIMEOUT}
    RETURN    ${response}

POST Request
    [Documentation]    執行 POST 請求（含 JSON body，會捕獲 HTTP 錯誤並返回響應）
    [Arguments]    ${url}    ${data}=${EMPTY}    ${headers}=${EMPTY}
    ${default_headers}=    Create Dictionary    Content-Type=application/json
    ${has_headers}=    Run Keyword And Return Status    Evaluate    '${headers}' != '${EMPTY}' and isinstance(${headers}, dict) if '${headers}' != '${EMPTY}' else False
    ${final_headers}=    Run Keyword If    ${has_headers}    Create Dictionary    &{default_headers}    &{headers}
    ...    ELSE    Set Variable    ${default_headers}
    # 將 Robot Framework 字典轉換為 Python dict：使用 Evaluate 確保正確傳遞
    ${json_data}=    Run Keyword If    '${data}' != '${EMPTY}'    Evaluate    ${data} if isinstance(${data}, dict) else {}
    ...    ELSE    Evaluate    {}
    ${response}=    POST Request With Error Handling    ${url}    json_data=${json_data}    headers=${final_headers}    timeout=${API_TIMEOUT}
    RETURN    ${response}

PUT Request
    [Documentation]    執行 PUT 請求（會捕獲 HTTP 錯誤並返回響應）
    [Arguments]    ${url}    ${data}=${EMPTY}    ${headers}=${EMPTY}
    ${default_headers}=    Create Dictionary    Content-Type=application/json
    ${has_headers}=    Run Keyword And Return Status    Evaluate    '${headers}' != '${EMPTY}' and isinstance(${headers}, dict) if '${headers}' != '${EMPTY}' else False
    ${final_headers}=    Run Keyword If    ${has_headers}    Create Dictionary    &{default_headers}    &{headers}
    ...    ELSE    Set Variable    ${default_headers}
    # 將 Robot Framework 字典轉換為 Python dict：使用 Evaluate 確保正確傳遞
    ${json_data}=    Run Keyword If    '${data}' != '${EMPTY}'    Evaluate    ${data} if isinstance(${data}, dict) else {}
    ...    ELSE    Evaluate    {}
    ${response}=    PUT Request With Error Handling    ${url}    json_data=${json_data}    headers=${final_headers}    timeout=${API_TIMEOUT}
    RETURN    ${response}

DELETE Request
    [Documentation]    執行 DELETE 請求（會捕獲 HTTP 錯誤並返回響應）
    [Arguments]    ${url}    ${headers}=${EMPTY}
    ${has_headers}=    Run Keyword And Return Status    Evaluate    '${headers}' != '${EMPTY}' and isinstance(${headers}, dict) if '${headers}' != '${EMPTY}' else False
    ${response}=    Run Keyword If    ${has_headers}    DELETE Request With Error Handling    ${url}    headers=${headers}    timeout=${API_TIMEOUT}
    ...    ELSE    DELETE Request With Error Handling    ${url}    timeout=${API_TIMEOUT}
    RETURN    ${response}

Validate Status Code
    [Documentation]    驗證 HTTP 狀態碼
    [Arguments]    ${response}    ${expected_status}
    ${status_str}=    Convert To String    ${expected_status}
    Status Should Be    ${status_str}    ${response}

Validate JSON Response
    [Documentation]    驗證 JSON 響應結構
    [Arguments]    ${response}
    ${json}=    Set Variable    ${response.json()}
    RETURN    ${json}

Create Session
    [Documentation]    建立 Requests 會話
    [Arguments]    ${alias}=api    ${url}=${API_BASE_URL}
    RequestsLibrary.Create Session    ${alias}    ${url}
