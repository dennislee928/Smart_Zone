import { Link } from 'react-router-dom'
import { useApplications, useDeleteApplication } from '../api/applications'
import type { Application } from '../types'

export default function ApplicationsList() {
  const { data: applications, isLoading, error } = useApplications()
  const deleteApplication = useDeleteApplication()

  if (isLoading) return <div>載入中...</div>
  if (error) return <div>錯誤：{String(error)}</div>

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold">申請列表</h1>
        <Link
          to="/applications/new"
          className="px-4 py-2 bg-primary text-primary-foreground rounded hover:opacity-90"
        >
          新增申請
        </Link>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {applications?.map((app) => (
          <ApplicationCard
            key={app.id}
            application={app}
            onDelete={() => app.id && deleteApplication.mutate(app.id)}
          />
        ))}
      </div>

      {applications?.length === 0 && (
        <div className="text-center py-8 text-muted-foreground">
          沒有申請記錄
        </div>
      )}
    </div>
  )
}

function ApplicationCard({ application, onDelete }: { application: Application; onDelete: () => void }) {
  return (
    <div className="p-6 bg-card border border-border rounded-lg">
      <div className="flex items-start justify-between mb-2">
        <Link
          to={`/applications/${application.id}`}
          className="text-lg font-semibold hover:text-primary"
        >
          {application.name}
        </Link>
        <button
          onClick={onDelete}
          className="text-destructive hover:underline text-sm"
        >
          刪除
        </button>
      </div>
      {application.deadline && (
        <p className="text-sm text-muted-foreground mb-2">截止日期：{application.deadline}</p>
      )}
      <p className="text-sm text-muted-foreground mb-2">狀態：{application.status}</p>
      {application.progress !== undefined && (
        <div className="mt-2">
          <div className="flex items-center justify-between text-sm mb-1">
            <span>進度</span>
            <span>{application.progress}%</span>
          </div>
          <div className="w-full bg-muted rounded-full h-2">
            <div
              className="bg-primary h-2 rounded-full"
              style={{ width: `${application.progress}%` }}
            />
          </div>
        </div>
      )}
    </div>
  )
}
