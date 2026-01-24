import { useState, useEffect } from 'react'
import { useCriteria, useUpdateCriteria } from '../api/criteria'

export default function CriteriaEditor() {
  const { data: criteria, isLoading, error } = useCriteria()
  const updateCriteria = useUpdateCriteria()
  const [formData, setFormData] = useState({
    required: '',
    preferred: '',
    excluded: '',
  })

  useEffect(() => {
    if (criteria?.criteriaJson) {
      setFormData({
        required: criteria.criteriaJson.required?.join('\n') || '',
        preferred: criteria.criteriaJson.preferred?.join('\n') || '',
        excluded: criteria.criteriaJson.excluded_keywords?.join('\n') || '',
      })
    }
  }, [criteria])

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      await updateCriteria.mutateAsync({
        criteriaJson: {
          required: formData.required.split('\n').filter(s => s.trim()),
          preferred: formData.preferred.split('\n').filter(s => s.trim()),
          excluded_keywords: formData.excluded.split('\n').filter(s => s.trim()),
        },
        profileJson: criteria?.profileJson,
      })
      alert('已更新搜尋條件')
    } catch (error) {
      console.error('Failed to update criteria:', error)
      alert('更新失敗')
    }
  }

  if (isLoading) return <div>載入中...</div>
  if (error) return <div>錯誤：{String(error)}</div>

  return (
    <div className="max-w-4xl mx-auto">
      <h1 className="text-3xl font-bold mb-6">搜尋條件設定</h1>
      <form onSubmit={handleSubmit} className="space-y-6">
        <div>
          <label className="block text-sm font-medium mb-2">必要條件（每行一個）</label>
          <textarea
            value={formData.required}
            onChange={(e) => setFormData({ ...formData, required: e.target.value })}
            className="w-full px-4 py-2 border border-border rounded"
            rows={6}
            placeholder="international&#10;master&#10;postgraduate"
          />
        </div>
        <div>
          <label className="block text-sm font-medium mb-2">偏好條件（每行一個）</label>
          <textarea
            value={formData.preferred}
            onChange={(e) => setFormData({ ...formData, preferred: e.target.value })}
            className="w-full px-4 py-2 border border-border rounded"
            rows={6}
            placeholder="Taiwanese eligible&#10;full tuition"
          />
        </div>
        <div>
          <label className="block text-sm font-medium mb-2">排除關鍵字（每行一個）</label>
          <textarea
            value={formData.excluded}
            onChange={(e) => setFormData({ ...formData, excluded: e.target.value })}
            className="w-full px-4 py-2 border border-border rounded"
            rows={6}
            placeholder="undergraduate only&#10;PhD only"
          />
        </div>
        <button
          type="submit"
          disabled={updateCriteria.isPending}
          className="px-6 py-2 bg-primary text-primary-foreground rounded hover:opacity-90 disabled:opacity-50"
        >
          {updateCriteria.isPending ? '更新中...' : '更新條件'}
        </button>
      </form>
    </div>
  )
}
