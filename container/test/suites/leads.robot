*** Settings ***
Documentation    Leads API 測試套件 - 完整 CRUD 操作和邊界情況
Resource          ../resources/api_keywords.robot
Resource          ../resources/validators.robot
Resource          ../resources/fixtures.robot
Variables         ${EXECDIR}/variables/config.robot
Suite Setup       Setup Test Database
Suite Teardown    Teardown Test Database
Test Setup        Create Session    api
Test Teardown     Delete All Sessions

*** Test Cases ***
Get All Leads Should Return Empty List
    [Documentation]    測試 GET /api/leads 返回空列表
    ${response}=    GET Request    ${API_LEADS}
    Response Should Have Status    ${response}    ${STATUS_OK}
    Lead List Should Be Empty    ${response}

Get All Leads With Status Filter
    [Documentation]    測試 GET /api/leads?status=qualified 篩選
    ${test_lead}=    Create Test Lead    status=qualified
    ${params}=    Create Dictionary    status=qualified
    ${response}=    GET Request    ${API_LEADS}    params=${params}
    Response Should Have Status    ${response}    ${STATUS_OK}
    ${json}=    Validate JSON Response    ${response}
    Should Not Be Empty    ${json}[leads]    Leads list should not be empty
    FOR    ${lead}    IN    @{json}[leads]
        Should Be Equal    ${lead}[status]    qualified    All leads should have status 'qualified'
    END

Get All Leads With Bucket Filter
    [Documentation]    測試 GET /api/leads?bucket=test-bucket 篩選
    ${test_lead}=    Create Test Lead    bucket=test-bucket
    ${params}=    Create Dictionary    bucket=test-bucket
    ${response}=    GET Request    ${API_LEADS}    params=${params}
    Response Should Have Status    ${response}    ${STATUS_OK}
    ${json}=    Validate JSON Response    ${response}
    Should Not Be Empty    ${json}[leads]    Leads list should not be empty

Get All Leads With Search Filter
    [Documentation]    測試 GET /api/leads?search=Test 搜尋
    ${test_lead}=    Create Test Lead    name=Test Scholarship Search
    ${params}=    Create Dictionary    search=Test
    ${response}=    GET Request    ${API_LEADS}    params=${params}
    Response Should Have Status    ${response}    ${STATUS_OK}
    ${json}=    Validate JSON Response    ${response}
    Should Not Be Empty    ${json}[leads]    Leads list should not be empty

Get Lead By Valid ID
    [Documentation]    測試 GET /api/leads/:id 取得存在的獎學金
    ${test_lead}=    Create Test Lead
    ${lead_id}=    Set Variable    ${test_lead}[id]
    ${response}=    GET Request    ${API_LEADS}/${lead_id}
    Response Should Have Status    ${response}    ${STATUS_OK}
    ${json}=    Validate JSON Response    ${response}
    Dictionary Should Contain Key    ${json}    lead
    Lead Should Match Schema    ${json}[lead]
    Should Be Equal    ${json}[lead][id]    ${lead_id}    Lead ID should match

Get Lead By Invalid ID Should Return 404
    [Documentation]    測試 GET /api/leads/:id 不存在的 ID（404）
    ${response}=    GET Request    ${API_LEADS}/99999
    Response Should Have Status    ${response}    ${STATUS_NOT_FOUND}
    Response Should Contain Error    ${response}    Lead not found

Get Lead By Non-Numeric ID Should Return 400
    [Documentation]    測試 GET /api/leads/:id 無效 ID（400）
    ${response}=    GET Request    ${API_LEADS}/invalid
    Response Should Have Status    ${response}    ${STATUS_BAD_REQUEST}
    Response Should Contain Error    ${response}    Invalid ID

Create Lead With Minimal Required Fields
    [Documentation]    測試 POST /api/leads 建立新獎學金（最小必填欄位）
    ${lead_data}=    Create Dictionary    name=Minimal Test Lead
    ${response}=    POST Request    ${API_LEADS}    ${lead_data}
    Response Should Have Status    ${response}    ${STATUS_CREATED}
    ${json}=    Validate JSON Response    ${response}
    Dictionary Should Contain Key    ${json}    lead
    Lead Should Match Schema    ${json}[lead]
    Should Be Equal    ${json}[lead][name]    Minimal Test Lead    Lead name should match

Create Lead With Full Data
    [Documentation]    測試 POST /api/leads 建立完整資料
    ${lead_data}=    Create Dictionary
    ...    name    Full Test Lead
    ...    amount    10000
    ...    deadline    2025-12-31
    ...    source    Test Source
    ...    status    qualified
    ...    bucket    test-bucket
    ...    notes    Test notes
    ${response}=    POST Request    ${API_LEADS}    ${lead_data}
    Response Should Have Status    ${response}    ${STATUS_CREATED}
    ${json}=    Validate JSON Response    ${response}
    Lead Should Match Schema    ${json}[lead]
    Should Be Equal    ${json}[lead][name]    Full Test Lead
    Should Be Equal    ${json}[lead][amount]    10000
    Should Be Equal    ${json}[lead][status]    qualified

Create Lead Without Required Field Should Return 400
    [Documentation]    測試 POST /api/leads 缺少必填欄位（400）
    ${lead_data}=    Create Dictionary    amount=5000
    ${response}=    POST Request    ${API_LEADS}    ${lead_data}
    Response Should Have Status    ${response}    ${STATUS_BAD_REQUEST}

Create Lead With Empty Name Should Return 400
    [Documentation]    測試 POST /api/leads 無效資料格式（400）
    ${lead_data}=    Create Dictionary    name=${EMPTY}
    ${response}=    POST Request    ${API_LEADS}    ${lead_data}
    Response Should Have Status    ${response}    ${STATUS_BAD_REQUEST}

Update Lead With Valid ID
    [Documentation]    測試 PUT /api/leads/:id 更新存在的獎學金
    ${test_lead}=    Create Test Lead    name=Original Name
    ${lead_id}=    Set Variable    ${test_lead}[id]
    ${update_data}=    Create Dictionary    name=Updated Name    amount=7500
    ${response}=    PUT Request    ${API_LEADS}/${lead_id}    ${update_data}
    Response Should Have Status    ${response}    ${STATUS_OK}
    ${json}=    Validate JSON Response    ${response}
    Should Be Equal    ${json}[lead][name]    Updated Name    Name should be updated
    Should Be Equal    ${json}[lead][amount]    7500    Amount should be updated

Update Lead With Partial Data
    [Documentation]    測試 PUT /api/leads/:id 部分更新
    ${test_lead}=    Create Test Lead    name=Original    amount=5000
    ${lead_id}=    Set Variable    ${test_lead}[id]
    ${update_data}=    Create Dictionary    amount=6000
    ${response}=    PUT Request    ${API_LEADS}/${lead_id}    ${update_data}
    Response Should Have Status    ${response}    ${STATUS_OK}
    ${json}=    Validate JSON Response    ${response}
    Should Be Equal    ${json}[lead][amount]    6000    Amount should be updated
    Should Be Equal    ${json}[lead][name]    Original    Name should remain unchanged

Update Lead With Invalid ID Should Return 404
    [Documentation]    測試 PUT /api/leads/:id 不存在的 ID（404）
    ${update_data}=    Create Dictionary    name=Updated Name
    ${response}=    PUT Request    ${API_LEADS}/99999    ${update_data}
    Response Should Have Status    ${response}    ${STATUS_NOT_FOUND}
    Response Should Contain Error    ${response}    Lead not found

Update Lead With Non-Numeric ID Should Return 400
    [Documentation]    測試 PUT /api/leads/:id 無效 ID（400）
    ${update_data}=    Create Dictionary    name=Updated Name
    ${response}=    PUT Request    ${API_LEADS}/invalid    ${update_data}
    Response Should Have Status    ${response}    ${STATUS_BAD_REQUEST}

Delete Lead With Valid ID
    [Documentation]    測試 DELETE /api/leads/:id 刪除存在的獎學金
    ${test_lead}=    Create Test Lead
    ${lead_id}=    Set Variable    ${test_lead}[id]
    ${response}=    DELETE Request    ${API_LEADS}/${lead_id}
    Response Should Have Status    ${response}    ${STATUS_OK}
    Response Should Contain Success    ${response}
    # 驗證已刪除
    ${get_response}=    GET Request    ${API_LEADS}/${lead_id}
    Response Should Have Status    ${get_response}    ${STATUS_NOT_FOUND}

Delete Lead With Invalid ID Should Return 404
    [Documentation]    測試 DELETE /api/leads/:id 不存在的 ID（404）
    ${response}=    DELETE Request    ${API_LEADS}/99999
    Response Should Have Status    ${response}    ${STATUS_NOT_FOUND}
    Response Should Contain Error    ${response}    Lead not found

Delete Lead With Non-Numeric ID Should Return 400
    [Documentation]    測試 DELETE /api/leads/:id 無效 ID（400）
    ${response}=    DELETE Request    ${API_LEADS}/invalid
    Response Should Have Status    ${response}    ${STATUS_BAD_REQUEST}
