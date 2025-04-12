"use client"

import Link from "next/link"
import { usePathname } from "next/navigation"
import { motion } from "framer-motion"
import { cn } from "@/lib/utils"
import { Home, BarChart2, Settings, CheckSquare } from "lucide-react"
import { sidebarVariants, sidebarItemVariants } from "@/animations/variants"

export function Sidebar() {
  const pathname = usePathname()

  const navItems = [
    { icon: Home, href: "/", label: "Home" },
    { icon: BarChart2, href: "/analytics", label: "Analytics" },
    { icon: CheckSquare, href: "/verification", label: "Verification" },
  ]

  return (
    <motion.div
      className="w-16 h-screen bg-[#111] border-r border-[#222] flex flex-col items-center py-4 flex-shrink-0"
      initial="hidden"
      animate="visible"
      variants={sidebarVariants}
    >
      <motion.div className="mb-8" variants={sidebarItemVariants}>
        <motion.div
          className="w-10 h-10 rounded-full bg-purple-600 flex items-center justify-center text-white font-bold"
          whileHover={{ scale: 1.1 }}
          whileTap={{ scale: 0.9 }}
        >
          A
        </motion.div>
      </motion.div>

      <nav className="flex-1 w-full">
        <ul className="flex flex-col items-center gap-4">
          {navItems.map((item, index) => {
            const isActive = pathname === item.href

            return (
              <motion.li
                key={item.href}
                className="w-full flex justify-center"
                variants={sidebarItemVariants}
                custom={index}
              >
                <motion.div whileHover={{ scale: 1.1 }} whileTap={{ scale: 0.9 }}>
                  <Link
                    href={item.href}
                    className={cn(
                      "w-10 h-10 flex items-center justify-center rounded-md transition-colors",
                      isActive ? "bg-[#333] text-white" : "text-[#777] hover:bg-[#222] hover:text-white",
                    )}
                    title={item.label}
                  >
                    <item.icon className="h-5 w-5" />
                  </Link>
                </motion.div>
              </motion.li>
            )
          })}
        </ul>
      </nav>

      <motion.div className="mt-auto" variants={sidebarItemVariants}>
        <motion.div whileHover={{ scale: 1.1 }} whileTap={{ scale: 0.9 }}>
          <button
            className="w-10 h-10 flex items-center justify-center rounded-md text-[#777] hover:bg-[#222] hover:text-white transition-colors"
            title="Settings"
          >
            <Settings className="h-5 w-5" />
          </button>
        </motion.div>
      </motion.div>
    </motion.div>
  )
}

