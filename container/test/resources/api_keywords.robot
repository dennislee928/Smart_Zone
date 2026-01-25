*** Settings ***
Documentation    可重用的 API 請求關鍵字庫
Library           RequestsLibrary
Library           Collections
Variables         ${EXECDIR}/variables/config.py

*** Keywords ***
GET Request
    [Documentation]    執行 GET 請求
    [Arguments]    ${url}    ${params}=${EMPTY}    ${headers}=${EMPTY}
    ${has_params}=    Run Keyword And Return Status    Evaluate    '${params}' != '${EMPTY}' and isinstance(${params}, dict) if '${params}' != '${EMPTY}' else False
    ${has_headers}=    Run Keyword And Return Status    Evaluate    '${headers}' != '${EMPTY}' and isinstance(${headers}, dict) if '${headers}' != '${EMPTY}' else False
    ${response}=    Run Keyword If    ${has_params} and ${has_headers}    GET    ${url}    params=${params}    headers=${headers}    timeout=${API_TIMEOUT}
    ...    ELSE IF    ${has_params}    GET    ${url}    params=${params}    timeout=${API_TIMEOUT}
    ...    ELSE IF    ${has_headers}    GET    ${url}    headers=${headers}    timeout=${API_TIMEOUT}
    ...    ELSE    GET    ${url}    timeout=${API_TIMEOUT}
    RETURN    ${response}

POST Request
    [Documentation]    執行 POST 請求（含 JSON body）
    [Arguments]    ${url}    ${data}=${EMPTY}    ${headers}=${EMPTY}
    ${is_dict}=    Run Keyword And Return Status    Evaluate    isinstance(${data}, dict) if '${data}' != '${EMPTY}' else False
    ${default_headers}=    Create Dictionary    Content-Type=application/json
    ${has_headers}=    Run Keyword And Return Status    Evaluate    '${headers}' != '${EMPTY}' and isinstance(${headers}, dict) if '${headers}' != '${EMPTY}' else False
    ${final_headers}=    Run Keyword If    ${has_headers}    Create Dictionary    &{default_headers}    &{headers}
    ...    ELSE    Set Variable    ${default_headers}
    ${response}=    Run Keyword If    ${is_dict}    POST    ${url}    json=${data}    headers=${final_headers}    timeout=${API_TIMEOUT}
    ...    ELSE    POST    ${url}    headers=${final_headers}    timeout=${API_TIMEOUT}
    RETURN    ${response}

PUT Request
    [Documentation]    執行 PUT 請求
    [Arguments]    ${url}    ${data}=${EMPTY}    ${headers}=${EMPTY}
    ${is_dict}=    Run Keyword And Return Status    Evaluate    isinstance(${data}, dict) if '${data}' != '${EMPTY}' else False
    ${default_headers}=    Create Dictionary    Content-Type=application/json
    ${has_headers}=    Run Keyword And Return Status    Evaluate    '${headers}' != '${EMPTY}' and isinstance(${headers}, dict) if '${headers}' != '${EMPTY}' else False
    ${final_headers}=    Run Keyword If    ${has_headers}    Create Dictionary    &{default_headers}    &{headers}
    ...    ELSE    Set Variable    ${default_headers}
    ${response}=    Run Keyword If    ${is_dict}    PUT    ${url}    json=${data}    headers=${final_headers}    timeout=${API_TIMEOUT}
    ...    ELSE    PUT    ${url}    headers=${final_headers}    timeout=${API_TIMEOUT}
    RETURN    ${response}

DELETE Request
    [Documentation]    執行 DELETE 請求
    [Arguments]    ${url}    ${headers}=${EMPTY}
    ${has_headers}=    Run Keyword And Return Status    Evaluate    '${headers}' != '${EMPTY}' and ${headers} is not None
    ${response}=    Run Keyword If    ${has_headers}    DELETE    ${url}    headers=${headers}    timeout=${API_TIMEOUT}
    ...    ELSE    DELETE    ${url}    timeout=${API_TIMEOUT}
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
