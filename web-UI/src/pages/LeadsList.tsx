import { useState } from 'react'
import { Link } from 'react-router-dom'
import { useLeads } from '../api/leads'
import type { Lead } from '../types'

export default function LeadsList() {
  const [statusFilter, setStatusFilter] = useState<string>('')
  const [bucketFilter, setBucketFilter] = useState<string>('')
  const [searchQuery, setSearchQuery] = useState<string>('')

  const { data: leads, isLoading, error } = useLeads({
    ...(statusFilter && { status: statusFilter }),
    ...(bucketFilter && { bucket: bucketFilter }),
    ...(searchQuery && { search: searchQuery }),
  })

  if (isLoading) return <div>載入中...</div>
  if (error) return <div>錯誤：{String(error)}</div>

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold">獎學金列表</h1>
      </div>

      <div className="flex gap-4 items-center">
        <input
          type="text"
          placeholder="搜尋..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="px-4 py-2 border border-border rounded"
        />
        <select
          value={statusFilter}
          onChange={(e) => setStatusFilter(e.target.value)}
          className="px-4 py-2 border border-border rounded"
        >
          <option value="">所有狀態</option>
          <option value="qualified">已符合</option>
          <option value="pending">待處理</option>
        </select>
        <select
          value={bucketFilter}
          onChange={(e) => setBucketFilter(e.target.value)}
          className="px-4 py-2 border border-border rounded"
        >
          <option value="">所有分類</option>
          <option value="A">A - 主攻</option>
          <option value="B">B - 備援</option>
          <option value="C">C - 淘汰</option>
        </select>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {leads?.map((lead) => (
          <LeadCard key={lead.id} lead={lead} />
        ))}
      </div>

      {leads?.length === 0 && (
        <div className="text-center py-8 text-muted-foreground">
          沒有找到獎學金
        </div>
      )}
    </div>
  )
}

function LeadCard({ lead }: { lead: Lead }) {
  return (
    <Link
      to={`/leads/${lead.id}`}
      className="p-6 bg-card border border-border rounded-lg hover:border-primary transition-colors"
    >
      <h3 className="text-lg font-semibold mb-2">{lead.name}</h3>
      {lead.amount && (
        <p className="text-sm text-muted-foreground mb-2">金額：{lead.amount}</p>
      )}
      {lead.deadline && (
        <p className="text-sm text-muted-foreground mb-2">截止日期：{lead.deadline}</p>
      )}
      {lead.matchScore && (
        <p className="text-sm text-muted-foreground">匹配分數：{lead.matchScore}</p>
      )}
      {lead.bucket && (
        <span className={`inline-block px-2 py-1 rounded text-xs mt-2 ${
          lead.bucket === 'A' ? 'bg-green-100 text-green-800' :
          lead.bucket === 'B' ? 'bg-yellow-100 text-yellow-800' :
          'bg-gray-100 text-gray-800'
        }`}>
          {lead.bucket}
        </span>
      )}
    </Link>
  )
}
