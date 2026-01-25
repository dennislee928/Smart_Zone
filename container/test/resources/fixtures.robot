*** Settings ***
Documentation    測試資料設置和清理
Resource          api_keywords.robot
Variables         ${EXECDIR}/variables/config.py
Library           Collections

*** Variables ***
@{CREATED_LEAD_IDS}    
@{CREATED_APPLICATION_IDS}    
${CREATED_CRITERIA_ID}    ${EMPTY}

*** Keywords ***
Setup Test Database
    [Documentation]    測試前清理資料庫（透過 API 刪除所有測試資料）
    Create Session    api
    # 清理所有 leads
    ${leads_response}=    GET Request    ${API_LEADS}
    Run Keyword If    ${leads_response.status_code} == ${STATUS_OK}
    ...    Cleanup All Leads    ${leads_response.json()}
    # 清理所有 applications
    ${apps_response}=    GET Request    ${API_APPLICATIONS}
    Run Keyword If    ${apps_response.status_code} == ${STATUS_OK}
    ...    Cleanup All Applications    ${apps_response.json()}
    # 清理 criteria（可選）
    ${criteria_response}=    GET Request    ${API_CRITERIA}
    Run Keyword If    ${criteria_response.status_code} == ${STATUS_OK} and ${criteria_response.json()}[criteria] != ${None}
    ...    Cleanup Criteria

Teardown Test Database
    [Documentation]    測試後清理資料庫
    Cleanup Created Leads
    Cleanup Created Applications
    Cleanup Created Criteria

Cleanup All Leads
    [Arguments]    ${leads_data}
    ${leads}=    Set Variable    ${leads_data}[leads]
    FOR    ${lead}    IN    @{leads}
        ${lead_id}=    Set Variable    ${lead}[id]
        DELETE Request    ${API_LEADS}/${lead_id}
    END

Cleanup All Applications
    [Arguments]    ${apps_data}
    ${applications}=    Set Variable    ${apps_data}[applications]
    FOR    ${app}    IN    @{applications}
        ${app_id}=    Set Variable    ${app}[id]
        DELETE Request    ${API_APPLICATIONS}/${app_id}
    END

Cleanup Created Leads
    [Documentation]    清理測試中建立的 leads
    FOR    ${lead_id}    IN    @{CREATED_LEAD_IDS}
        DELETE Request    ${API_LEADS}/${lead_id}
    END
    Set Suite Variable    @{CREATED_LEAD_IDS}    @{EMPTY}

Cleanup Created Applications
    [Documentation]    清理測試中建立的 applications
    FOR    ${app_id}    IN    @{CREATED_APPLICATION_IDS}
        DELETE Request    ${API_APPLICATIONS}/${app_id}
    END
    Set Suite Variable    @{CREATED_APPLICATION_IDS}    @{EMPTY}

Cleanup Criteria
    [Documentation]    清理 criteria（透過 PUT 設定為 null）
    # 發送 null 值來清空 criteria
    ${empty_criteria}=    Evaluate    {'criteriaJson': None, 'profileJson': None}
    PUT Request    ${API_CRITERIA}    ${empty_criteria}

Cleanup Created Criteria
    [Documentation]    清理測試中建立的 criteria
    Run Keyword If    '${CREATED_CRITERIA_ID}' != '${EMPTY}'    Cleanup Criteria
    Set Suite Variable    ${CREATED_CRITERIA_ID}    ${EMPTY}

Create Test Lead
    [Documentation]    建立測試用獎學金資料
    [Arguments]    ${name}=${TEST_LEAD_NAME}    ${amount}=${TEST_LEAD_AMOUNT}    ${status}=${TEST_LEAD_STATUS}    ${bucket}=${TEST_LEAD_BUCKET}    &{extra_fields}
    ${lead_data}=    Create Dictionary    name=${name}    amount=${amount}    status=${status}    bucket=${bucket}
    Set To Dictionary    ${lead_data}    &{extra_fields}
    ${response}=    POST Request    ${API_LEADS}    ${lead_data}
    Validate Status Code    ${response}    ${STATUS_CREATED}
    ${json}=    Validate JSON Response    ${response}
    ${lead_id}=    Set Variable    ${json}[lead][id]
    Append To List    ${CREATED_LEAD_IDS}    ${lead_id}
    RETURN    ${json}[lead]

Create Test Application
    [Documentation]    建立測試用申請資料
    [Arguments]    ${name}=${TEST_APPLICATION_NAME}    ${status}=${TEST_APPLICATION_STATUS}    &{extra_fields}
    ${app_data}=    Create Dictionary    name=${name}    status=${status}
    Set To Dictionary    ${app_data}    &{extra_fields}
    ${response}=    POST Request    ${API_APPLICATIONS}    ${app_data}
    Validate Status Code    ${response}    ${STATUS_CREATED}
    ${json}=    Validate JSON Response    ${response}
    ${app_id}=    Set Variable    ${json}[application][id]
    Append To List    ${CREATED_APPLICATION_IDS}    ${app_id}
    RETURN    ${json}[application]

Create Test Criteria
    [Documentation]    建立測試用搜尋條件
    [Arguments]    &{criteria_data}
    ${default_criteria}=    Create Dictionary
    ...    criteriaJson    ${{{"required": [], "preferred": [], "excluded_keywords": []}}}
    ...    profileJson    ${{{"nationality": "TW", "target_university": "Test University", "target_country": "US", "programme_level": "Master", "programme_start": "2025", "education": []}}}
    Set To Dictionary    ${default_criteria}    &{criteria_data}
    ${response}=    PUT Request    ${API_CRITERIA}    ${default_criteria}
    Validate Status Code    ${response}    ${STATUS_OK}
    ${json}=    Validate JSON Response    ${response}
    Set Suite Variable    ${CREATED_CRITERIA_ID}    ${json}[criteria][id]
    RETURN    ${json}[criteria]
