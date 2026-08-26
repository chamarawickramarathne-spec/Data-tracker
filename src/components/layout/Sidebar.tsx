import { useAppStore } from '@/stores/appStore'
import type { Page } from '@/types'
import { Activity, Calendar, CalendarDays, Settings, Gauge, Grid3x3 } from 'lucide-react'

const navItems: Array<{ id: Page; label: string; icon: React.ComponentType<any> }> = [
  { id: 'dashboard', label: 'Dashboard', icon: Activity },
  { id: 'daily', label: 'Daily Usage', icon: Calendar },
  { id: 'monthly', label: 'Monthly Usage', icon: CalendarDays },
  { id: 'peakhours', label: 'Peak Hours', icon: Grid3x3 },
  { id: 'speedtest', label: 'Speed Test', icon: Gauge },
  { id: 'settings', label: 'Settings', icon: Settings },
]

export function Sidebar() {
  const { currentPage, setCurrentPage } = useAppStore()

  return (
    <aside className="w-56 bg-card border-r border-border flex flex-col">
      <nav className="flex-1 p-3 space-y-1">
        {navItems.map((item) => {
          const Icon = item.icon
          const isActive = currentPage === item.id
          return (
            <button
              key={item.id}
              onClick={() => setCurrentPage(item.id)}
              className={`
                w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-all
                ${isActive
                  ? 'bg-primary text-primary-foreground shadow-md'
                  : 'text-muted-foreground hover:bg-muted hover:text-foreground'
                }
              `}
            >
              <Icon className="w-4 h-4" />
              {item.label}
            </button>
          )
        })}
      </nav>
    </aside>
  )
}
