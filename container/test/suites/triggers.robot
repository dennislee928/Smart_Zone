*** Settings ***
Documentation    Triggers API 測試套件
Resource          ../resources/api_keywords.robot
Resource          ../resources/validators.robot
Variables         variables/config.robot
Suite Setup       Create Session    api
Suite Teardown    Delete All Sessions

*** Test Cases ***
Trigger Search Should Return Accepted
    [Documentation]    測試 POST /api/trigger/search 觸發 Rust 爬蟲（驗證 202 狀態碼和響應結構）
    ${response}=    POST Request    ${API_TRIGGER_SEARCH}
    Response Should Have Status    ${response}    ${STATUS_ACCEPTED}
    Response Should Be Valid JSON    ${response}
    ${json}=    Validate JSON Response    ${response}
    Dictionary Should Contain Key    ${json}    message
    Dictionary Should Contain Key    ${json}    status
    Dictionary Should Contain Key    ${json}    note
    Should Be Equal    ${json}[status]    pending    Status should be 'pending'
    Should Contain    ${json}[message]    Search trigger received    Message should contain expected text

Trigger Schedule Should Return Accepted
    [Documentation]    測試 POST /api/trigger/schedule 觸發 Go 排程建議（驗證 202 狀態碼和響應結構）
    ${response}=    POST Request    ${API_TRIGGER_SCHEDULE}
    Response Should Have Status    ${response}    ${STATUS_ACCEPTED}
    Response Should Be Valid JSON    ${response}
    ${json}=    Validate JSON Response    ${response}
    Dictionary Should Contain Key    ${json}    message
    Dictionary Should Contain Key    ${json}    status
    Dictionary Should Contain Key    ${json}    note
    Should Be Equal    ${json}[status]    pending    Status should be 'pending'
    Should Contain    ${json}[message]    Schedule trigger received    Message should contain expected text

Trigger Track Should Return Accepted
    [Documentation]    測試 POST /api/trigger/track 觸發 Go 進度追蹤（驗證 202 狀態碼和響應結構）
    ${response}=    POST Request    ${API_TRIGGER_TRACK}
    Response Should Have Status    ${response}    ${STATUS_ACCEPTED}
    Response Should Be Valid JSON    ${response}
    ${json}=    Validate JSON Response    ${response}
    Dictionary Should Contain Key    ${json}    message
    Dictionary Should Contain Key    ${json}    status
    Dictionary Should Contain Key    ${json}    note
    Should Be Equal    ${json}[status]    pending    Status should be 'pending'
    Should Contain    ${json}[message]    Track trigger received    Message should contain expected text

Trigger Search Response Should Contain Note
    [Documentation]    驗證 search trigger 響應包含 note 欄位
    ${response}=    POST Request    ${API_TRIGGER_SEARCH}
    ${json}=    Validate JSON Response    ${response}
    Should Not Be Empty    ${json}[note]    Note field should not be empty

Trigger Schedule Response Should Contain Note
    [Documentation]    驗證 schedule trigger 響應包含 note 欄位
    ${response}=    POST Request    ${API_TRIGGER_SCHEDULE}
    ${json}=    Validate JSON Response    ${response}
    Should Not Be Empty    ${json}[note]    Note field should not be empty

Trigger Track Response Should Contain Note
    [Documentation]    驗證 track trigger 響應包含 note 欄位
    ${response}=    POST Request    ${API_TRIGGER_TRACK}
    ${json}=    Validate JSON Response    ${response}
    Should Not Be Empty    ${json}[note]    Note field should not be empty
