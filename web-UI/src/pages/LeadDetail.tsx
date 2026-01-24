import { useParams, Link } from 'react-router-dom'
import { useLead } from '../api/leads'

export default function LeadDetail() {
  const { id } = useParams<{ id: string }>()
  const leadId = id ? parseInt(id) : 0
  const { data: lead, isLoading, error } = useLead(leadId)

  if (isLoading) return <div>載入中...</div>
  if (error) return <div>錯誤：{String(error)}</div>
  if (!lead) return <div>找不到獎學金</div>

  return (
    <div className="space-y-6">
      <Link to="/leads" className="text-primary hover:underline">
        ← 返回列表
      </Link>

      <div className="p-6 bg-card border border-border rounded-lg">
        <h1 className="text-3xl font-bold mb-4">{lead.name}</h1>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {lead.amount && (
            <div>
              <h3 className="font-semibold text-muted-foreground">金額</h3>
              <p>{lead.amount}</p>
            </div>
          )}
          {lead.deadline && (
            <div>
              <h3 className="font-semibold text-muted-foreground">截止日期</h3>
              <p>{lead.deadline}</p>
            </div>
          )}
          {lead.source && (
            <div>
              <h3 className="font-semibold text-muted-foreground">來源</h3>
              <a href={lead.source} target="_blank" rel="noopener noreferrer" className="text-primary hover:underline">
                {lead.source}
              </a>
            </div>
          )}
          {lead.status && (
            <div>
              <h3 className="font-semibold text-muted-foreground">狀態</h3>
              <p>{lead.status}</p>
            </div>
          )}
          {lead.matchScore !== undefined && (
            <div>
              <h3 className="font-semibold text-muted-foreground">匹配分數</h3>
              <p>{lead.matchScore}</p>
            </div>
          )}
          {lead.bucket && (
            <div>
              <h3 className="font-semibold text-muted-foreground">分類</h3>
              <p>{lead.bucket}</p>
            </div>
          )}
        </div>

        {lead.eligibility && lead.eligibility.length > 0 && (
          <div className="mt-4">
            <h3 className="font-semibold text-muted-foreground mb-2">資格要求</h3>
            <ul className="list-disc list-inside">
              {lead.eligibility.map((req, i) => (
                <li key={i}>{req}</li>
              ))}
            </ul>
          </div>
        )}

        {lead.matchReasons && lead.matchReasons.length > 0 && (
          <div className="mt-4">
            <h3 className="font-semibold text-muted-foreground mb-2">匹配原因</h3>
            <ul className="list-disc list-inside">
              {lead.matchReasons.map((reason, i) => (
                <li key={i}>{reason}</li>
              ))}
            </ul>
          </div>
        )}

        {lead.notes && (
          <div className="mt-4">
            <h3 className="font-semibold text-muted-foreground mb-2">備註</h3>
            <p>{lead.notes}</p>
          </div>
        )}
      </div>
    </div>
  )
}
