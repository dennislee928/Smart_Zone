*** Settings ***
Documentation    可重用的 API 請求關鍵字庫
Library           RequestsLibrary
Variables         ${EXECDIR}/variables/config.py

*** Keywords ***
GET Request
    [Documentation]    執行 GET 請求
    [Arguments]    ${url}    ${params}=${EMPTY}    ${headers}=${EMPTY}
    ${response}=    Run Keyword If    '${params}' != '${EMPTY}'    GET    ${url}    params=${params}    headers=${headers}    timeout=${API_TIMEOUT}
    ...    ELSE IF    '${headers}' != '${EMPTY}'    GET    ${url}    headers=${headers}    timeout=${API_TIMEOUT}
    ...    ELSE    GET    ${url}    timeout=${API_TIMEOUT}
    RETURN    ${response}

POST Request
    [Documentation]    執行 POST 請求（含 JSON body）
    [Arguments]    ${url}    ${data}=${EMPTY}    ${headers}=${EMPTY}
    ${json_data}=    Run Keyword If    '${data}' != '${EMPTY}'    Evaluate    json.dumps(${data})    json
    ...    ELSE    Set Variable    ${EMPTY}
    ${default_headers}=    Create Dictionary    Content-Type=application/json
    ${final_headers}=    Run Keyword If    '${headers}' != '${EMPTY}'    Create Dictionary    &{default_headers}    &{headers}
    ...    ELSE    Set Variable    ${default_headers}
    ${response}=    Run Keyword If    '${json_data}' != '${EMPTY}'    POST    ${url}    json=${data}    headers=${final_headers}    timeout=${API_TIMEOUT}
    ...    ELSE    POST    ${url}    headers=${final_headers}    timeout=${API_TIMEOUT}
    RETURN    ${response}

PUT Request
    [Documentation]    執行 PUT 請求
    [Arguments]    ${url}    ${data}=${EMPTY}    ${headers}=${EMPTY}
    ${default_headers}=    Create Dictionary    Content-Type=application/json
    ${final_headers}=    Run Keyword If    '${headers}' != '${EMPTY}'    Create Dictionary    &{default_headers}    &{headers}
    ...    ELSE    Set Variable    ${default_headers}
    ${response}=    Run Keyword If    '${data}' != '${EMPTY}'    PUT    ${url}    json=${data}    headers=${final_headers}    timeout=${API_TIMEOUT}
    ...    ELSE    PUT    ${url}    headers=${final_headers}    timeout=${API_TIMEOUT}
    RETURN    ${response}

DELETE Request
    [Documentation]    執行 DELETE 請求
    [Arguments]    ${url}    ${headers}=${EMPTY}
    ${response}=    Run Keyword If    '${headers}' != '${EMPTY}'    DELETE    ${url}    headers=${headers}    timeout=${API_TIMEOUT}
    ...    ELSE    DELETE    ${url}    timeout=${API_TIMEOUT}
    RETURN    ${response}

Validate Status Code
    [Documentation]    驗證 HTTP 狀態碼
    [Arguments]    ${response}    ${expected_status}
    Status Should Be    ${expected_status}    ${response}

Validate JSON Response
    [Documentation]    驗證 JSON 響應結構
    [Arguments]    ${response}
    ${json}=    Set Variable    ${response.json()}
    RETURN    ${json}

Create Session
    [Documentation]    建立 Requests 會話
    [Arguments]    ${alias}=api    ${url}=${API_BASE_URL}
    Create Session    ${alias}    ${url}
