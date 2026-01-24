import { useParams, Link, useNavigate } from 'react-router-dom'
import { useApplication, useUpdateApplication, useDeleteApplication } from '../api/applications'
import { useState } from 'react'

export default function ApplicationDetail() {
  const { id } = useParams<{ id: string }>()
  const leadId = id ? parseInt(id) : 0
  const navigate = useNavigate()
  const { data: application, isLoading, error } = useApplication(leadId)
  const updateApplication = useUpdateApplication()
  const deleteApplication = useDeleteApplication()
  const [isEditing, setIsEditing] = useState(false)
  const [formData, setFormData] = useState({
    name: '',
    deadline: '',
    status: 'not_started',
    currentStage: '',
    nextAction: '',
    requiredDocs: '',
    progress: 0,
    notes: '',
  })

  if (isLoading) return <div>載入中...</div>
  if (error) return <div>錯誤：{String(error)}</div>
  if (!application) return <div>找不到申請</div>

  if (isEditing && !formData.name) {
    setFormData({
      name: application.name || '',
      deadline: application.deadline || '',
      status: application.status,
      currentStage: application.currentStage || '',
      nextAction: application.nextAction || '',
      requiredDocs: application.requiredDocs?.join(', ') || '',
      progress: application.progress || 0,
      notes: application.notes || '',
    })
  }

  const handleSave = async () => {
    if (!application.id) return
    try {
      await updateApplication.mutateAsync({
        id: application.id,
        ...formData,
        requiredDocs: formData.requiredDocs ? formData.requiredDocs.split(',').map(s => s.trim()) : [],
        progress: Number(formData.progress),
      })
      setIsEditing(false)
    } catch (error) {
      console.error('Failed to update application:', error)
    }
  }

  const handleDelete = async () => {
    if (!application.id) return
    if (confirm('確定要刪除這個申請嗎？')) {
      try {
        await deleteApplication.mutateAsync(application.id)
        navigate('/applications')
      } catch (error) {
        console.error('Failed to delete application:', error)
      }
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <Link to="/applications" className="text-primary hover:underline">
          ← 返回列表
        </Link>
        <div className="flex gap-2">
          {isEditing ? (
            <>
              <button
                onClick={handleSave}
                disabled={updateApplication.isPending}
                className="px-4 py-2 bg-primary text-primary-foreground rounded hover:opacity-90 disabled:opacity-50"
              >
                儲存
              </button>
              <button
                onClick={() => setIsEditing(false)}
                className="px-4 py-2 border border-border rounded hover:bg-muted"
              >
                取消
              </button>
            </>
          ) : (
            <>
              <button
                onClick={() => setIsEditing(true)}
                className="px-4 py-2 bg-primary text-primary-foreground rounded hover:opacity-90"
              >
                編輯
              </button>
              <button
                onClick={handleDelete}
                disabled={deleteApplication.isPending}
                className="px-4 py-2 bg-destructive text-destructive-foreground rounded hover:opacity-90 disabled:opacity-50"
              >
                刪除
              </button>
            </>
          )}
        </div>
      </div>

      <div className="p-6 bg-card border border-border rounded-lg">
        {isEditing ? (
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium mb-1">名稱</label>
              <input
                type="text"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                className="w-full px-4 py-2 border border-border rounded"
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">截止日期</label>
              <input
                type="date"
                value={formData.deadline}
                onChange={(e) => setFormData({ ...formData, deadline: e.target.value })}
                className="w-full px-4 py-2 border border-border rounded"
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">狀態</label>
              <select
                value={formData.status}
                onChange={(e) => setFormData({ ...formData, status: e.target.value })}
                className="w-full px-4 py-2 border border-border rounded"
              >
                <option value="not_started">未開始</option>
                <option value="in_progress">進行中</option>
                <option value="submitted">已提交</option>
                <option value="accepted">已接受</option>
                <option value="rejected">已拒絕</option>
              </select>
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">進度 (%)</label>
              <input
                type="number"
                min="0"
                max="100"
                value={formData.progress}
                onChange={(e) => setFormData({ ...formData, progress: Number(e.target.value) })}
                className="w-full px-4 py-2 border border-border rounded"
              />
            </div>
          </div>
        ) : (
          <>
            <h1 className="text-3xl font-bold mb-4">{application.name}</h1>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {application.deadline && (
                <div>
                  <h3 className="font-semibold text-muted-foreground">截止日期</h3>
                  <p>{application.deadline}</p>
                </div>
              )}
              <div>
                <h3 className="font-semibold text-muted-foreground">狀態</h3>
                <p>{application.status}</p>
              </div>
              {application.progress !== undefined && (
                <div>
                  <h3 className="font-semibold text-muted-foreground">進度</h3>
                  <p>{application.progress}%</p>
                </div>
              )}
            </div>
            {application.notes && (
              <div className="mt-4">
                <h3 className="font-semibold text-muted-foreground mb-2">備註</h3>
                <p>{application.notes}</p>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  )
}
