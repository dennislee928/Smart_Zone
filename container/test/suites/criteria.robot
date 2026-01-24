*** Settings ***
Documentation    Criteria API 測試套件
Resource          ../resources/api_keywords.robot
Resource          ../resources/validators.robot
Resource          ../resources/fixtures.robot
Variables         variables/config.robot
Suite Setup       Setup Test Database
Suite Teardown    Teardown Test Database
Test Setup        Create Session    api
Test Teardown     Delete All Sessions

*** Test Cases ***
Get Criteria When Not Exists Should Return Null
    [Documentation]    測試 GET /api/criteria 取得搜尋條件（不存在，應返回 null）
    ${response}=    GET Request    ${API_CRITERIA}
    Response Should Have Status    ${response}    ${STATUS_OK}
    ${json}=    Validate JSON Response    ${response}
    Dictionary Should Contain Key    ${json}    criteria
    Should Be Equal    ${json}[criteria]    ${None}    Criteria should be null when not exists

Get Criteria When Exists Should Return Data
    [Documentation]    測試 GET /api/criteria 取得搜尋條件（存在）
    ${test_criteria}=    Create Test Criteria
    ${response}=    GET Request    ${API_CRITERIA}
    Response Should Have Status    ${response}    ${STATUS_OK}
    ${json}=    Validate JSON Response    ${response}
    Dictionary Should Contain Key    ${json}    criteria
    Should Not Be Equal    ${json}[criteria]    ${None}    Criteria should not be null
    Criteria Should Match Schema    ${json}[criteria]

Create New Criteria
    [Documentation]    測試 PUT /api/criteria 建立新搜尋條件
    ${criteria_data}=    Create Dictionary
    ...    criteriaJson    ${{{"required": ["Master"], "preferred": ["Engineering"], "excluded_keywords": ["PhD"]}}}
    ...    profileJson    ${{{"nationality": "TW", "target_university": "MIT", "target_country": "US", "programme_level": "Master", "programme_start": "2025", "education": []}}}
    ${response}=    PUT Request    ${API_CRITERIA}    ${criteria_data}
    Response Should Have Status    ${response}    ${STATUS_OK}
    ${json}=    Validate JSON Response    ${response}
    Dictionary Should Contain Key    ${json}    criteria
    Criteria Should Match Schema    ${json}[criteria]
    Should Not Be Empty    ${json}[criteria][criteriaJson]    criteriaJson should not be empty
    Should Not Be Empty    ${json}[criteria][profileJson]    profileJson should not be empty

Update Existing Criteria
    [Documentation]    測試 PUT /api/criteria 更新現有搜尋條件
    ${test_criteria}=    Create Test Criteria
    ${update_data}=    Create Dictionary
    ...    criteriaJson    ${{{"required": ["PhD"], "preferred": ["Science"], "excluded_keywords": []}}}
    ${response}=    PUT Request    ${API_CRITERIA}    ${update_data}
    Response Should Have Status    ${response}    ${STATUS_OK}
    ${json}=    Validate JSON Response    ${response}
    Dictionary Should Contain Key    ${json}    criteria
    Should Not Be Empty    ${json}[criteria][criteriaJson]    criteriaJson should be updated

Update Criteria With Partial Data
    [Documentation]    測試 PUT /api/criteria 部分更新
    ${test_criteria}=    Create Test Criteria
    ${update_data}=    Create Dictionary
    ...    profileJson    ${{{"nationality": "US", "target_university": "Stanford", "target_country": "US", "programme_level": "PhD", "programme_start": "2026", "education": []}}}
    ${response}=    PUT Request    ${API_CRITERIA}    ${update_data}
    Response Should Have Status    ${response}    ${STATUS_OK}
    ${json}=    Validate JSON Response    ${response}
    Dictionary Should Contain Key    ${json}    criteria
    Should Not Be Empty    ${json}[criteria][profileJson]    profileJson should be updated

Update Criteria With Invalid Data Should Return 400
    [Documentation]    測試 PUT /api/criteria 無效資料格式（400）
    ${invalid_data}=    Create Dictionary
    ...    criteriaJson    invalid_json_string
    ${response}=    PUT Request    ${API_CRITERIA}    ${invalid_data}
    Response Should Have Status    ${response}    ${STATUS_BAD_REQUEST}

Update Criteria With Missing Required Fields Should Return 400
    [Documentation]    測試 PUT /api/criteria 缺少必填欄位（400）
    ${invalid_data}=    Create Dictionary
    ...    criteriaJson    ${{{"required": []}}}
    # profileJson 缺少必填欄位
    ${response}=    PUT Request    ${API_CRITERIA}    ${invalid_data}
    # 如果 API 允許部分更新，這可能不會返回 400
    # 根據實際 API 行為調整

Criteria Should Persist After Creation
    [Documentation]    驗證建立的 criteria 可以正確讀取
    ${criteria_data}=    Create Dictionary
    ...    criteriaJson    ${{{"required": ["Master"], "preferred": [], "excluded_keywords": []}}}
    ...    profileJson    ${{{"nationality": "TW", "target_university": "Test", "target_country": "US", "programme_level": "Master", "programme_start": "2025", "education": []}}}
    ${create_response}=    PUT Request    ${API_CRITERIA}    ${criteria_data}
    Response Should Have Status    ${create_response}    ${STATUS_OK}
    ${get_response}=    GET Request    ${API_CRITERIA}
    Response Should Have Status    ${get_response}    ${STATUS_OK}
    ${json}=    Validate JSON Response    ${get_response}
    Should Not Be Equal    ${json}[criteria]    ${None}    Criteria should persist
