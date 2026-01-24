*** Settings ***
Documentation    Health Check API 測試套件
Resource          ../resources/api_keywords.robot
Resource          ../resources/validators.robot
Variables         ${EXECDIR}/variables/config.robot
Suite Setup       Create Session    api
Suite Teardown    Delete All Sessions

*** Test Cases ***
Health Check Should Return OK
    [Documentation]    測試 GET / 端點返回正確的狀態
    ${response}=    GET Request    ${API_HEALTH}
    Response Should Have Status    ${response}    ${STATUS_OK}
    Response Should Be Valid JSON    ${response}
    Response Should Contain Field    ${response}    $.message
    Response Should Contain Field    ${response}    $.version
    Response Should Contain Field    ${response}    $.status
    ${json}=    Validate JSON Response    ${response}
    Should Be Equal    ${json}[status]    ok    Status should be 'ok'

Health Check Should Contain API Message
    [Documentation]    驗證 health check 響應包含 API 訊息
    ${response}=    GET Request    ${API_HEALTH}
    ${json}=    Validate JSON Response    ${response}
    Should Contain    ${json}[message]    ScholarshipOps    Message should contain 'ScholarshipOps'

Health Check Should Have Version
    [Documentation]    驗證 health check 響應包含版本號
    ${response}=    GET Request    ${API_HEALTH}
    ${json}=    Validate JSON Response    ${response}
    Should Match Regexp    ${json}[version]    \\d+\\.\\d+\\.\\d+    Version should match semantic versioning
