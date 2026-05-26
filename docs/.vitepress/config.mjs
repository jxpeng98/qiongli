const enNav = [
  { text: 'Guide', link: '/guide/' },
  { text: 'Workflows', link: '/guide/task-recipes' },
  { text: 'Examples', link: '/examples/' },
  { text: 'CLI', link: '/reference/cli' },
  { text: 'Architecture', link: '/architecture' },
  { text: 'Advanced', link: '/advanced/' },
  { text: 'Maintainer', link: '/maintainer/' }
]

const zhNav = [
  { text: '入门', link: '/zh/guide/' },
  { text: '工作流', link: '/zh/guide/task-recipes' },
  { text: '示例', link: '/zh/examples/' },
  { text: 'CLI', link: '/zh/reference/cli' },
  { text: '架构', link: '/zh/architecture' },
  { text: '高级', link: '/zh/advanced/' },
  { text: '维护者', link: '/zh/maintainer/' }
]

const enSidebar = {
  '/guide/': [
    {
      text: 'Guide',
      items: [
        { text: 'Overview', link: '/guide/' },
        { text: 'Quickstart', link: '/quickstart' },
        { text: 'Install', link: '/guide/install' },
        { text: 'Using Agent Skills', link: '/guide/using-agent-skills' },
        { text: 'Research Workflows', link: '/guide/task-recipes' },
        { text: 'Multi-Agent Runtime', link: '/guide/multi-agent' },
        { text: 'Upgrade', link: '/guide/upgrade' },
        { text: 'Troubleshooting', link: '/guide/troubleshooting' }
      ]
    }
  ],
  '/reference/': [
    {
      text: 'Reference',
      items: [
        { text: 'Overview', link: '/reference/' },
        { text: 'CLI Reference', link: '/reference/cli' },
        { text: 'Skills Guide', link: '/reference/skills' },
        { text: 'Conventions', link: '/conventions' }
      ]
    }
  ],
  '/examples/': [
    {
      text: 'Examples',
      items: [
        { text: 'Overview', link: '/examples/' },
        { text: 'Paper Type Playbooks', link: '/examples/paper-type-playbooks' }
      ]
    }
  ],
  '/advanced/': [
    {
      text: 'Advanced',
      items: [
        { text: 'Overview', link: '/advanced/' },
        { text: 'Extend Qiongli', link: '/advanced/extend-qiongli' },
        { text: 'Agent + Skill Collaboration', link: '/advanced/agent-skill-collaboration' },
        { text: 'Plugin-First Architecture', link: '/advanced/plugin-first-architecture' },
        { text: 'MCP Providers Setup', link: '/advanced/mcp-providers-setup' },
        { text: 'Rigorous Literature Search', link: '/advanced/rigorous-literature-search' },
        { text: 'Zotero Integration', link: '/advanced/mcp-zotero-integration' },
        { text: 'Publish to PyPI', link: '/advanced/publish-pypi' }
      ]
    }
  ],
  '/maintainer/': [
    {
      text: 'Maintainer',
      items: [
        { text: 'Overview', link: '/maintainer/' },
        { text: 'CLAUDE Guide Summary', link: '/maintainer/claude-overview' },
        { text: 'Architecture', link: '/architecture' },
        { text: 'Conventions', link: '/conventions' },
        { text: 'Naming Policy', link: '/maintainer/naming-policy' },
        { text: 'Release Branch Policy', link: '/maintainer/release-branch-policy' },
        { text: 'Publish to PyPI', link: '/advanced/publish-pypi' }
      ]
    }
  ]
}

const zhSidebar = {
  '/zh/guide/': [
    {
      text: '入门',
      items: [
        { text: '总览', link: '/zh/guide/' },
        { text: '快速开始', link: '/zh/quickstart' },
        { text: '安装', link: '/zh/guide/install' },
        { text: '使用 Agent Skills', link: '/zh/guide/using-agent-skills' },
        { text: '研究工作流', link: '/zh/guide/task-recipes' },
        { text: '多 Agent 运行', link: '/zh/guide/multi-agent' },
        { text: '升级', link: '/zh/guide/upgrade' },
        { text: '故障排除', link: '/zh/guide/troubleshooting' }
      ]
    }
  ],
  '/zh/reference/': [
    {
      text: '参考',
      items: [
        { text: '总览', link: '/zh/reference/' },
        { text: 'CLI 参考', link: '/zh/reference/cli' },
        { text: 'Skills 指南', link: '/zh/reference/skills' },
        { text: '规范约定', link: '/zh/conventions' }
      ]
    }
  ],
  '/zh/examples/': [
    {
      text: '示例',
      items: [
        { text: '总览', link: '/zh/examples/' },
        { text: 'Paper Type 路线图', link: '/zh/examples/paper-type-playbooks' }
      ]
    }
  ],
  '/zh/advanced/': [
    {
      text: '高级',
      items: [
        { text: '总览', link: '/zh/advanced/' },
        { text: '扩展 Qiongli', link: '/zh/advanced/extend-qiongli' },
        { text: 'Agent + Skill 协同', link: '/zh/advanced/agent-skill-collaboration' },
        { text: 'Plugin-First 架构', link: '/advanced/plugin-first-architecture' },
        { text: 'MCP Providers 接入', link: '/zh/advanced/mcp-providers-setup' },
        { text: '严格 Literature Search', link: '/zh/advanced/rigorous-literature-search' },
        { text: 'Zotero 集成', link: '/zh/advanced/mcp-zotero-integration' },
        { text: '发布到 PyPI', link: '/zh/advanced/publish-pypi' }
      ]
    }
  ],
  '/zh/maintainer/': [
    {
      text: '维护者',
      items: [
        { text: '总览', link: '/zh/maintainer/' },
        { text: 'CLAUDE 指南摘要', link: '/zh/maintainer/claude-overview' },
        { text: '系统架构', link: '/zh/architecture' },
        { text: '规范约定', link: '/zh/conventions' },
        { text: '命名策略', link: '/zh/maintainer/naming-policy' },
        { text: '发布分支策略', link: '/zh/maintainer/release-branch-policy' },
        { text: 'PyPI 发布', link: '/zh/advanced/publish-pypi' }
      ]
    }
  ]
}

const commonHead = [
  ['meta', { name: 'theme-color', content: '#0f766e' }],
  ['meta', { name: 'author', content: 'Jiaxin Peng' }],
  ['link', { rel: 'icon', type: 'image/svg+xml', href: '/mark.svg' }]
]

/** @type {import('vitepress').UserConfig} */
export default {
  title: 'Qiongli',
  description: 'Contract-driven academic research workflows for Codex, Claude Code, and Gemini.',
  cleanUrls: true,
  lastUpdated: true,
  head: commonHead,
  ignoreDeadLinks: [/^https?:\/\//],
  locales: {
    root: {
      label: 'English',
      lang: 'en-GB',
      themeConfig: {
        nav: enNav,
        sidebar: enSidebar,
        search: { provider: 'local' },
        outline: { level: [2, 3] },
        socialLinks: [{ icon: 'github', link: 'https://github.com/jxpeng98/qiongli' }],
        footer: {
          message: 'Qiongli documentation',
          copyright: 'MIT License'
        }
      }
    },
    zh: {
      label: '简体中文',
      lang: 'zh-CN',
      link: '/zh/',
      themeConfig: {
        nav: zhNav,
        sidebar: zhSidebar,
        search: { provider: 'local' },
        outline: { level: [2, 3] },
        socialLinks: [{ icon: 'github', link: 'https://github.com/jxpeng98/qiongli' }],
        footer: {
          message: 'Qiongli 文档站',
          copyright: 'MIT License'
        }
      }
    }
  },
  themeConfig: {
    logo: { src: '/mark.svg', alt: 'Qiongli' }
  }
}
