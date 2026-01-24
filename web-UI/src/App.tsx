import { Routes, Route, Link } from 'react-router-dom'
import Dashboard from './pages/Dashboard'
import LeadsList from './pages/LeadsList'
import LeadDetail from './pages/LeadDetail'
import ApplicationsList from './pages/ApplicationsList'
import ApplicationForm from './pages/ApplicationForm'
import ApplicationDetail from './pages/ApplicationDetail'
import CriteriaEditor from './pages/CriteriaEditor'

function App() {
  return (
    <div className="min-h-screen bg-background">
      <nav className="border-b border-border bg-card">
        <div className="container mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            <Link to="/" className="text-xl font-bold text-foreground">
              ScholarshipOps
            </Link>
            <div className="flex gap-4">
              <Link to="/" className="text-foreground hover:text-primary">
                儀表板
              </Link>
              <Link to="/leads" className="text-foreground hover:text-primary">
                獎學金
              </Link>
              <Link to="/applications" className="text-foreground hover:text-primary">
                申請
              </Link>
              <Link to="/criteria" className="text-foreground hover:text-primary">
                搜尋條件
              </Link>
            </div>
          </div>
        </div>
      </nav>

      <main className="container mx-auto px-4 py-8">
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/leads" element={<LeadsList />} />
          <Route path="/leads/:id" element={<LeadDetail />} />
          <Route path="/applications" element={<ApplicationsList />} />
          <Route path="/applications/new" element={<ApplicationForm />} />
          <Route path="/applications/:id" element={<ApplicationDetail />} />
          <Route path="/criteria" element={<CriteriaEditor />} />
        </Routes>
      </main>
    </div>
  )
}

export default App
