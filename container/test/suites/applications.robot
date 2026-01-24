*** Settings ***
Documentation    Applications API 測試套件 - 完整 CRUD 操作和邊界情況
Resource          ../resources/api_keywords.robot
Resource          ../resources/validators.robot
Resource          ../resources/fixtures.robot
Variables         ${CURDIR}/../variables/config.robot
Suite Setup       Setup Test Database
Suite Teardown    Teardown Test Database
Test Setup        Create Session    api
Test Teardown     Delete All Sessions

*** Test Cases ***
Get All Applications Should Return Empty List
    [Documentation]    測試 GET /api/applications 返回空列表
    ${response}=    GET Request    ${API_APPLICATIONS}
    Response Should Have Status    ${response}    ${STATUS_OK}
    Application List Should Be Empty    ${response}

Get Application By Valid ID
    [Documentation]    測試 GET /api/applications/:id 取得存在的申請
    ${test_app}=    Create Test Application
    ${app_id}=    Set Variable    ${test_app}[id]
    ${response}=    GET Request    ${API_APPLICATIONS}/${app_id}
    Response Should Have Status    ${response}    ${STATUS_OK}
    ${json}=    Validate JSON Response    ${response}
    Dictionary Should Contain Key    ${json}    application
    Application Should Match Schema    ${json}[application]
    Should Be Equal    ${json}[application][id]    ${app_id}    Application ID should match

Get Application By Invalid ID Should Return 404
    [Documentation]    測試 GET /api/applications/:id 不存在的 ID（404）
    ${response}=    GET Request    ${API_APPLICATIONS}/99999
    Response Should Have Status    ${response}    ${STATUS_NOT_FOUND}
    Response Should Contain Error    ${response}    Application not found

Get Application By Non-Numeric ID Should Return 400
    [Documentation]    測試 GET /api/applications/:id 無效 ID（400）
    ${response}=    GET Request    ${API_APPLICATIONS}/invalid
    Response Should Have Status    ${response}    ${STATUS_BAD_REQUEST}
    Response Should Contain Error    ${response}    Invalid ID

Create Application With Minimal Required Fields
    [Documentation]    測試 POST /api/applications 建立新申請（最小必填欄位）
    ${app_data}=    Create Dictionary    name=Minimal Test Application
    ${response}=    POST Request    ${API_APPLICATIONS}    ${app_data}
    Response Should Have Status    ${response}    ${STATUS_CREATED}
    ${json}=    Validate JSON Response    ${response}
    Dictionary Should Contain Key    ${json}    application
    Application Should Match Schema    ${json}[application]
    Should Be Equal    ${json}[application][name]    Minimal Test Application    Application name should match

Create Application With Full Data
    [Documentation]    測試 POST /api/applications 建立完整資料
    ${app_data}=    Create Dictionary
    ...    name    Full Test Application
    ...    deadline    2025-12-31
    ...    status    in_progress
    ...    currentStage    Application Submitted
    ...    nextAction    Wait for response
    ...    requiredDocs    ${{["Transcript", "Recommendation"]}}
    ...    progress    50
    ...    notes    Test notes
    ${response}=    POST Request    ${API_APPLICATIONS}    ${app_data}
    Response Should Have Status    ${response}    ${STATUS_CREATED}
    ${json}=    Validate JSON Response    ${response}
    Application Should Match Schema    ${json}[application]
    Should Be Equal    ${json}[application][name]    Full Test Application
    Should Be Equal    ${json}[application][status]    in_progress
    Should Be Equal    ${json}[application][progress]    ${50}

Create Application Without Required Field Should Return 400
    [Documentation]    測試 POST /api/applications 缺少必填欄位（400）
    ${app_data}=    Create Dictionary    status=in_progress
    ${response}=    POST Request    ${API_APPLICATIONS}    ${app_data}
    Response Should Have Status    ${response}    ${STATUS_BAD_REQUEST}

Create Application With Empty Name Should Return 400
    [Documentation]    測試 POST /api/applications 無效資料格式（400）
    ${app_data}=    Create Dictionary    name=${EMPTY}
    ${response}=    POST Request    ${API_APPLICATIONS}    ${app_data}
    Response Should Have Status    ${response}    ${STATUS_BAD_REQUEST}

Create Application With Invalid Progress Should Return 400
    [Documentation]    測試 POST /api/applications 無效 progress 值（400）
    ${app_data}=    Create Dictionary    name=Test App    progress=150
    ${response}=    POST Request    ${API_APPLICATIONS}    ${app_data}
    Response Should Have Status    ${response}    ${STATUS_BAD_REQUEST}

Update Application With Valid ID
    [Documentation]    測試 PUT /api/applications/:id 更新存在的申請
    ${test_app}=    Create Test Application    name=Original Name
    ${app_id}=    Set Variable    ${test_app}[id]
    ${update_data}=    Create Dictionary    name=Updated Name    progress=75
    ${response}=    PUT Request    ${API_APPLICATIONS}/${app_id}    ${update_data}
    Response Should Have Status    ${response}    ${STATUS_OK}
    ${json}=    Validate JSON Response    ${response}
    Should Be Equal    ${json}[application][name]    Updated Name    Name should be updated
    Should Be Equal    ${json}[application][progress]    ${75}    Progress should be updated

Update Application With Partial Data
    [Documentation]    測試 PUT /api/applications/:id 部分更新
    ${test_app}=    Create Test Application    name=Original    progress=50
    ${app_id}=    Set Variable    ${test_app}[id]
    ${update_data}=    Create Dictionary    progress=60
    ${response}=    PUT Request    ${API_APPLICATIONS}/${app_id}    ${update_data}
    Response Should Have Status    ${response}    ${STATUS_OK}
    ${json}=    Validate JSON Response    ${response}
    Should Be Equal    ${json}[application][progress]    ${60}    Progress should be updated
    Should Be Equal    ${json}[application][name]    Original    Name should remain unchanged

Update Application With Invalid ID Should Return 404
    [Documentation]    測試 PUT /api/applications/:id 不存在的 ID（404）
    ${update_data}=    Create Dictionary    name=Updated Name
    ${response}=    PUT Request    ${API_APPLICATIONS}/99999    ${update_data}
    Response Should Have Status    ${response}    ${STATUS_NOT_FOUND}
    Response Should Contain Error    ${response}    Application not found

Update Application With Non-Numeric ID Should Return 400
    [Documentation]    測試 PUT /api/applications/:id 無效 ID（400）
    ${update_data}=    Create Dictionary    name=Updated Name
    ${response}=    PUT Request    ${API_APPLICATIONS}/invalid    ${update_data}
    Response Should Have Status    ${response}    ${STATUS_BAD_REQUEST}

Delete Application With Valid ID
    [Documentation]    測試 DELETE /api/applications/:id 刪除存在的申請
    ${test_app}=    Create Test Application
    ${app_id}=    Set Variable    ${test_app}[id]
    ${response}=    DELETE Request    ${API_APPLICATIONS}/${app_id}
    Response Should Have Status    ${response}    ${STATUS_OK}
    Response Should Contain Success    ${response}
    # 驗證已刪除
    ${get_response}=    GET Request    ${API_APPLICATIONS}/${app_id}
    Response Should Have Status    ${get_response}    ${STATUS_NOT_FOUND}

Delete Application With Invalid ID Should Return 404
    [Documentation]    測試 DELETE /api/applications/:id 不存在的 ID（404）
    ${response}=    DELETE Request    ${API_APPLICATIONS}/99999
    Response Should Have Status    ${response}    ${STATUS_NOT_FOUND}
    Response Should Contain Error    ${response}    Application not found

Delete Application With Non-Numeric ID Should Return 400
    [Documentation]    測試 DELETE /api/applications/:id 無效 ID（400）
    ${response}=    DELETE Request    ${API_APPLICATIONS}/invalid
    Response Should Have Status    ${response}    ${STATUS_BAD_REQUEST}
