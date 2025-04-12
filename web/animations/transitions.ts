// 常用过渡配置

// 平滑的页面过渡
export const pageTransition = {
  duration: 0.4,
  ease: [0.25, 0.1, 0.25, 1.0], // 自定义缓动，使动作更平滑
}

// 快速的悬停过渡
export const hoverTransition = {
  duration: 0.2,
}

// 弹簧过渡，用于自然运动
export const springTransition = {
  type: "spring",
  stiffness: 100,
  damping: 15,
}

