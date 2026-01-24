import { useStats } from '../api/stats'
import { useTriggerSearch, useTriggerSchedule, useTriggerTrack } from '../api/triggers'

export default function Dashboard() {
  const { data: stats, isLoading, error } = useStats()
  const triggerSearch = useTriggerSearch()
  const triggerSchedule = useTriggerSchedule()
  const triggerTrack = useTriggerTrack()

  if (isLoading) return <div>載入中...</div>
  if (error) return <div>錯誤：{String(error)}</div>
  if (!stats) return <div>無資料</div>

  return (
    <div className="space-y-6">
      <h1 className="text-3xl font-bold">儀表板</h1>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <div className="p-6 bg-card border border-border rounded-lg">
          <h3 className="text-sm font-medium text-muted-foreground">總獎學金數</h3>
          <p className="text-2xl font-bold mt-2">{stats.totalLeads}</p>
        </div>
        <div className="p-6 bg-card border border-border rounded-lg">
          <h3 className="text-sm font-medium text-muted-foreground">總申請數</h3>
          <p className="text-2xl font-bold mt-2">{stats.totalApplications}</p>
        </div>
        <div className="p-6 bg-card border border-border rounded-lg">
          <h3 className="text-sm font-medium text-muted-foreground">進行中</h3>
          <p className="text-2xl font-bold mt-2">{stats.inProgress}</p>
        </div>
        <div className="p-6 bg-card border border-border rounded-lg">
          <h3 className="text-sm font-medium text-muted-foreground">已完成</h3>
          <p className="text-2xl font-bold mt-2">{stats.completed}</p>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="p-6 bg-card border border-border rounded-lg">
          <h3 className="text-sm font-medium text-muted-foreground">D-7 即將到期</h3>
          <p className="text-2xl font-bold mt-2 text-destructive">{stats.upcoming7}</p>
        </div>
        <div className="p-6 bg-card border border-border rounded-lg">
          <h3 className="text-sm font-medium text-muted-foreground">D-14 即將到期</h3>
          <p className="text-2xl font-bold mt-2">{stats.upcoming14}</p>
        </div>
        <div className="p-6 bg-card border border-border rounded-lg">
          <h3 className="text-sm font-medium text-muted-foreground">D-21 即將到期</h3>
          <p className="text-2xl font-bold mt-2">{stats.upcoming21}</p>
        </div>
      </div>

      <div className="p-6 bg-card border border-border rounded-lg">
        <h2 className="text-xl font-bold mb-4">觸發腳本</h2>
        <div className="flex gap-4">
          <button
            onClick={() => triggerSearch.mutate()}
            disabled={triggerSearch.isPending}
            className="px-4 py-2 bg-primary text-primary-foreground rounded hover:opacity-90 disabled:opacity-50"
          >
            {triggerSearch.isPending ? '執行中...' : '觸發搜尋'}
          </button>
          <button
            onClick={() => triggerSchedule.mutate()}
            disabled={triggerSchedule.isPending}
            className="px-4 py-2 bg-primary text-primary-foreground rounded hover:opacity-90 disabled:opacity-50"
          >
            {triggerSchedule.isPending ? '執行中...' : '觸發排程'}
          </button>
          <button
            onClick={() => triggerTrack.mutate()}
            disabled={triggerTrack.isPending}
            className="px-4 py-2 bg-primary text-primary-foreground rounded hover:opacity-90 disabled:opacity-50"
          >
            {triggerTrack.isPending ? '執行中...' : '觸發追蹤'}
          </button>
        </div>
      </div>
    </div>
  )
}
