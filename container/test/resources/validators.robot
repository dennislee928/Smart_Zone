*** Settings ***
Documentation    響應驗證關鍵字
Resource          api_keywords.robot
Library           Collections
Library           String

*** Keywords ***
Get Value From Json
    [Documentation]    從 JSON 物件中取得欄位值（支援簡單 JSONPath 語法如 $.field 或 $.nested.field）
    [Arguments]    ${json}    ${field_path}
    ${path}=    Remove String    ${field_path}    $    .
    ${path_parts}=    Split String    ${path}    .
    ${value}=    Set Variable    ${json}
    FOR    ${part}    IN    @{path_parts}
        ${value}=    Get From Dictionary    ${value}    ${part}
    END
    RETURN    ${value}

Response Should Contain Field
    [Documentation]    驗證響應包含特定欄位
    [Arguments]    ${response}    ${field_path}
    ${json}=    Validate JSON Response    ${response}
    ${field_value}=    Get Value From Json    ${json}    ${field_path}
    Should Not Be Empty    ${field_value}    Field ${field_path} not found in response

Response Should Have Status
    [Documentation]    驗證狀態碼
    [Arguments]    ${response}    ${expected_status}
    Validate Status Code    ${response}    ${expected_status}

Response Should Be Valid JSON
    [Documentation]    驗證 JSON 格式
    [Arguments]    ${response}
    ${json}=    Validate JSON Response    ${response}
    Should Not Be Empty    ${json}    Response is not valid JSON

Lead Should Match Schema
    [Documentation]    驗證 Lead 資料結構
    [Arguments]    ${lead_data}
    Dictionary Should Contain Key    ${lead_data}    id
    Dictionary Should Contain Key    ${lead_data}    name
    ${id_value}=    Get From Dictionary    ${lead_data}    id
    ${id_type}=    Evaluate    type(${id_value}).__name__
    Should Be Equal    ${id_type}    int    Lead ID should be integer
    ${name_value}=    Get From Dictionary    ${lead_data}    name
    ${name_type}=    Evaluate    type(${name_value}).__name__
    Should Be Equal    ${name_type}    str    Lead name should be string
    Run Keyword If    'status' in ${{list(${lead_data}.keys())}}
    ...    Evaluate    type(${lead_data}[status]).__name__ == 'str'    Lead status should be string

Application Should Match Schema
    [Documentation]    驗證 Application 資料結構
    [Arguments]    ${app_data}
    Dictionary Should Contain Key    ${app_data}    id
    Dictionary Should Contain Key    ${app_data}    name
    ${id_value}=    Get From Dictionary    ${app_data}    id
    ${id_type}=    Evaluate    type(${id_value}).__name__
    Should Be Equal    ${id_type}    int    Application ID should be integer
    ${name_value}=    Get From Dictionary    ${app_data}    name
    ${name_type}=    Evaluate    type(${name_value}).__name__
    Should Be Equal    ${name_type}    str    Application name should be string
    Run Keyword If    'status' in ${{list(${app_data}.keys())}}
    ...    Evaluate    type(${app_data}[status]).__name__ == 'str'    Application status should be string

Criteria Should Match Schema
    [Documentation]    驗證 Criteria 資料結構
    [Arguments]    ${criteria_data}
    Dictionary Should Contain Key    ${criteria_data}    id
    Run Keyword If    'criteriaJson' in ${{list(${criteria_data}.keys())}}
    ...    Dictionary Should Contain Key    ${criteria_data}    criteriaJson
    Run Keyword If    'profileJson' in ${{list(${criteria_data}.keys())}}
    ...    Dictionary Should Contain Key    ${criteria_data}    profileJson

Stats Should Match Schema
    [Documentation]    驗證 Stats 資料結構
    [Arguments]    ${stats_data}
    Should Not Be Empty    ${stats_data}    Stats data should not be empty
    # Stats 結構可能包含各種統計欄位，這裡只驗證基本結構
    ${stats_type}=    Evaluate    type(${stats_data}).__name__
    Should Be Equal    ${stats_type}    dict    Stats should be a dictionary

Response Should Contain Error
    [Documentation]    驗證錯誤響應
    [Arguments]    ${response}    ${error_message}=${EMPTY}
    ${json}=    Validate JSON Response    ${response}
    Dictionary Should Contain Key    ${json}    error
    Run Keyword If    '${error_message}' != '${EMPTY}'    Should Contain    ${json}[error]    ${error_message}

Response Should Contain Success
    [Documentation]    驗證成功響應
    [Arguments]    ${response}
    ${json}=    Validate JSON Response    ${response}
    Run Keyword If    'success' in ${{list(${json}.keys())}}    Should Be True    ${json}[success]    Success should be true

Lead List Should Be Empty
    [Documentation]    驗證 Lead 列表為空
    [Arguments]    ${response}
    ${json}=    Validate JSON Response    ${response}
    Dictionary Should Contain Key    ${json}    leads
    Should Be Empty    ${json}[leads]    Leads list should be empty

Application List Should Be Empty
    [Documentation]    驗證 Application 列表為空
    [Arguments]    ${response}
    ${json}=    Validate JSON Response    ${response}
    Dictionary Should Contain Key    ${json}    applications
    Should Be Empty    ${json}[applications]    Applications list should be empty
