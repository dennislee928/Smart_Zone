import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useCreateApplication } from '../api/applications'

export default function ApplicationForm() {
  const navigate = useNavigate()
  const createApplication = useCreateApplication()
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

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      await createApplication.mutateAsync({
        ...formData,
        requiredDocs: formData.requiredDocs ? formData.requiredDocs.split(',').map(s => s.trim()) : [],
        progress: Number(formData.progress),
      })
      navigate('/applications')
    } catch (error) {
      console.error('Failed to create application:', error)
    }
  }

  return (
    <div className="max-w-2xl mx-auto">
      <h1 className="text-3xl font-bold mb-6">新增申請</h1>
      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <label className="block text-sm font-medium mb-1">名稱 *</label>
          <input
            type="text"
            required
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
          <label className="block text-sm font-medium mb-1">目前階段</label>
          <input
            type="text"
            value={formData.currentStage}
            onChange={(e) => setFormData({ ...formData, currentStage: e.target.value })}
            className="w-full px-4 py-2 border border-border rounded"
          />
        </div>
        <div>
          <label className="block text-sm font-medium mb-1">下一步行動</label>
          <input
            type="text"
            value={formData.nextAction}
            onChange={(e) => setFormData({ ...formData, nextAction: e.target.value })}
            className="w-full px-4 py-2 border border-border rounded"
          />
        </div>
        <div>
          <label className="block text-sm font-medium mb-1">所需文件（逗號分隔）</label>
          <input
            type="text"
            value={formData.requiredDocs}
            onChange={(e) => setFormData({ ...formData, requiredDocs: e.target.value })}
            className="w-full px-4 py-2 border border-border rounded"
            placeholder="文件1, 文件2, 文件3"
          />
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
        <div>
          <label className="block text-sm font-medium mb-1">備註</label>
          <textarea
            value={formData.notes}
            onChange={(e) => setFormData({ ...formData, notes: e.target.value })}
            className="w-full px-4 py-2 border border-border rounded"
            rows={4}
          />
        </div>
        <div className="flex gap-4">
          <button
            type="submit"
            disabled={createApplication.isPending}
            className="px-6 py-2 bg-primary text-primary-foreground rounded hover:opacity-90 disabled:opacity-50"
          >
            {createApplication.isPending ? '建立中...' : '建立'}
          </button>
          <button
            type="button"
            onClick={() => navigate('/applications')}
            className="px-6 py-2 border border-border rounded hover:bg-muted"
          >
            取消
          </button>
        </div>
      </form>
    </div>
  )
}
