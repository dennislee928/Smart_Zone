*** Settings ***
Documentation    Stats API 測試套件
Resource          ../resources/api_keywords.robot
Resource          ../resources/validators.robot
Resource          ../resources/fixtures.robot
Variables         ${EXECDIR}/variables/config.robot
Suite Setup       Setup Test Database
Suite Teardown    Teardown Test Database
Test Setup        Create Session    api
Test Teardown     Delete All Sessions

*** Test Cases ***
Get Stats Should Return All Fields
    [Documentation]    測試 GET /api/stats 取得統計資料並驗證所有欄位存在
    ${response}=    GET Request    ${API_STATS}
    Response Should Have Status    ${response}    ${STATUS_OK}
    ${json}=    Validate JSON Response    ${response}
    Dictionary Should Contain Key    ${json}    stats
    Stats Should Match Schema    ${json}[stats]
    ${stats}=    Set Variable    ${json}[stats]
    Dictionary Should Contain Key    ${stats}    totalLeads
    Dictionary Should Contain Key    ${stats}    totalApplications
    Dictionary Should Contain Key    ${stats}    inProgress
    Dictionary Should Contain Key    ${stats}    completed
    Dictionary Should Contain Key    ${stats}    notStarted
    Dictionary Should Contain Key    ${stats}    upcoming7
    Dictionary Should Contain Key    ${stats}    upcoming14
    Dictionary Should Contain Key    ${stats}    upcoming21

Get Stats Should Return Correct Numeric Types
    [Documentation]    驗證統計數值類型正確
    ${response}=    GET Request    ${API_STATS}
    ${json}=    Validate JSON Response    ${response}
    ${stats}=    Set Variable    ${json}[stats]
    Should Be True    isinstance(${stats}[totalLeads], int)    totalLeads should be integer
    Should Be True    isinstance(${stats}[totalApplications], int)    totalApplications should be integer
    Should Be True    isinstance(${stats}[inProgress], int)    inProgress should be integer
    Should Be True    isinstance(${stats}[completed], int)    completed should be integer
    Should Be True    isinstance(${stats}[notStarted], int)    notStarted should be integer
    Should Be True    isinstance(${stats}[upcoming7], int)    upcoming7 should be integer
    Should Be True    isinstance(${stats}[upcoming14], int)    upcoming14 should be integer
    Should Be True    isinstance(${stats}[upcoming21], int)    upcoming21 should be integer

Get Stats With Empty Database Should Return Zeros
    [Documentation]    測試空資料庫時統計應返回零值
    ${response}=    GET Request    ${API_STATS}
    ${json}=    Validate JSON Response    ${response}
    ${stats}=    Set Variable    ${json}[stats]
    Should Be Equal    ${stats}[totalLeads]    ${0}    totalLeads should be 0
    Should Be Equal    ${stats}[totalApplications]    ${0}    totalApplications should be 0
    Should Be Equal    ${stats}[inProgress]    ${0}    inProgress should be 0
    Should Be Equal    ${stats}[completed]    ${0}    completed should be 0
    Should Be Equal    ${stats}[notStarted]    ${0}    notStarted should be 0

Get Stats Should Reflect Created Leads
    [Documentation]    測試建立 leads 後統計應正確反映
    ${test_lead1}=    Create Test Lead
    ${test_lead2}=    Create Test Lead
    ${response}=    GET Request    ${API_STATS}
    ${json}=    Validate JSON Response    ${response}
    ${stats}=    Set Variable    ${json}[stats]
    Should Be Equal    ${stats}[totalLeads]    ${2}    totalLeads should be 2

Get Stats Should Reflect Created Applications
    [Documentation]    測試建立 applications 後統計應正確反映
    ${test_app1}=    Create Test Application    status=not_started
    ${test_app2}=    Create Test Application    status=in_progress
    ${test_app3}=    Create Test Application    status=submitted
    ${response}=    GET Request    ${API_STATS}
    ${json}=    Validate JSON Response    ${response}
    ${stats}=    Set Variable    ${json}[stats]
    Should Be Equal    ${stats}[totalApplications]    ${3}    totalApplications should be 3
    Should Be Equal    ${stats}[notStarted]    ${1}    notStarted should be 1
    Should Be Equal    ${stats}[inProgress]    ${1}    inProgress should be 1
    Should Be Equal    ${stats}[completed]    ${1}    completed should be 1

Get Stats Should Calculate Upcoming Deadlines
    [Documentation]    測試統計應正確計算即將到來的截止日期
    # 建立未來 5 天、10 天、15 天的申請
    ${date_5_days}=    Evaluate    (datetime.datetime.now() + datetime.timedelta(days=5)).strftime('%Y-%m-%d')    modules=datetime
    ${date_10_days}=    Evaluate    (datetime.datetime.now() + datetime.timedelta(days=10)).strftime('%Y-%m-%d')    modules=datetime
    ${date_15_days}=    Evaluate    (datetime.datetime.now() + datetime.timedelta(days=15)).strftime('%Y-%m-%d')    modules=datetime
    ${test_app1}=    Create Test Application    deadline=${date_5_days}
    ${test_app2}=    Create Test Application    deadline=${date_10_days}
    ${test_app3}=    Create Test Application    deadline=${date_15_days}
    ${response}=    GET Request    ${API_STATS}
    ${json}=    Validate JSON Response    ${response}
    ${stats}=    Set Variable    ${json}[stats]
    Should Be True    ${stats}[upcoming7] >= ${1}    Should have at least 1 deadline in 7 days
    Should Be True    ${stats}[upcoming14] >= ${1}    Should have at least 1 deadline in 14 days
    Should Be True    ${stats}[upcoming21] >= ${1}    Should have at least 1 deadline in 21 days

Stats Should Be Non-Negative
    [Documentation]    驗證所有統計值應為非負數
    ${response}=    GET Request    ${API_STATS}
    ${json}=    Validate JSON Response    ${response}
    ${stats}=    Set Variable    ${json}[stats]
    Should Be True    ${stats}[totalLeads] >= ${0}    totalLeads should be non-negative
    Should Be True    ${stats}[totalApplications] >= ${0}    totalApplications should be non-negative
    Should Be True    ${stats}[inProgress] >= ${0}    inProgress should be non-negative
    Should Be True    ${stats}[completed] >= ${0}    completed should be non-negative
    Should Be True    ${stats}[notStarted] >= ${0}    notStarted should be non-negative
    Should Be True    ${stats}[upcoming7] >= ${0}    upcoming7 should be non-negative
    Should Be True    ${stats}[upcoming14] >= ${0}    upcoming14 should be non-negative
    Should Be True    ${stats}[upcoming21] >= ${0}    upcoming21 should be non-negative
