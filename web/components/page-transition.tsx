"use client"

import type { ReactNode } from "react"
import { motion } from "framer-motion"
import { pageTransition } from "@/animations/transitions"

interface PageTransitionProps {
  children: ReactNode
}

export function PageTransition({ children }: PageTransitionProps) {
  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={pageTransition}
      className="w-full"
    >
      {children}
    </motion.div>
  )
}

