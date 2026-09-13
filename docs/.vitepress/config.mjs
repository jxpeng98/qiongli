const enNav = [
  { text: 'Download 2.x', link: '/guide/cli-2x' },
  { text: 'Guide', link: '/guide/' },
  { text: 'Workflows', link: '/guide/task-recipes' },
  { text: 'Examples', link: '/examples/' },
  { text: 'CLI', link: '/guide/cli-2x' },
  { text: 'Architecture', link: '/architecture' },
  { text: 'Advanced', link: '/advanced/' },
  { text: 'Maintainer', link: '/maintainer/' }
]

const zhNav = [
  { text: '下载 2.x', link: '/zh/guide/cli-2x' },
  { text: '入门', link: '/zh/guide/' },
  { text: '工作流', link: '/zh/guide/task-recipes' },
  { text: '示例', link: '/zh/examples/' },
  { text: 'CLI', link: '/zh/guide/cli-2x' },
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
        { text: 'Install', link: '/guide/cli-2x#standalone-binary-download' },
        { text: '2.x CLI Downloads', link: '/guide/cli-2x' },
        { text: 'Using Agent Skills', link: '/guide/using-agent-skills' },
        { text: 'Research Workflows', link: '/guide/task-recipes' },
        { text: 'Work with other agents', link: '/guide/multi-agent' },
        { text: 'Upgrade', link: '/guide/cli-2x#install-and-upgrade-bundled-content' },
        { text: 'Data Ownership and Lifecycle', link: '/guide/data-lifecycle' },
        { text: 'Troubleshooting', link: '/guide/troubleshooting' }
      ]
    }
  ],
  '/reference/': [
    {
      text: 'Reference',
      items: [
        { text: 'Overview', link: '/reference/' },
        { text: '2.x CLI', link: '/guide/cli-2x' },
        { text: '1.x CLI (legacy)', link: '/reference/cli' },
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
        { text: 'Paper Type Playbooks', link: '/examples/paper-type-playbooks' },
        { text: 'Research Graph example', link: '/examples/research-graph' }
      ]
    }
  ],
  '/advanced/': [
    {
      text: 'Advanced',
      items: [
        { text: 'Overview', link: '/advanced/' },
        { text: 'Extend Qiongli', link: '/advanced/extend-qiongli' },
        { text: 'Subject Packaging Model', link: '/advanced/subject-packaging-model' },
        { text: 'Agents and optional hooks', link: '/advanced/agent-skill-collaboration' },
        { text: 'How the Plugin connects', link: '/advanced/plugin-first-architecture' },
        { text: 'MCP Providers Setup', link: '/advanced/mcp-providers-setup' },
        { text: 'Rigorous Literature Search', link: '/advanced/rigorous-literature-search' },
        { text: 'Zotero Integration', link: '/advanced/mcp-zotero-integration' },
        { text: 'Publish native packages', link: '/advanced/publish-pypi' }
      ]
    }
  ],
  '/maintainer/': [
    {
      text: 'Maintainer',
      items: [
        { text: 'Overview', link: '/maintainer/' },
        { text: 'Maintainer workflow', link: '/maintainer/claude-overview' },
        { text: 'Architecture', link: '/architecture' },
        { text: 'Conventions', link: '/conventions' },
        { text: 'Local Desktop Development', link: '/development/local-desktop-build' },
        { text: 'Repository Structure', link: '/development/repository-structure' },
        { text: 'Naming Policy', link: '/maintainer/naming-policy' },
        { text: 'External Borrowing', link: '/maintainer/external-borrowing' },
        { text: 'Release Branch Policy', link: '/maintainer/release-branch-policy' },
        { text: 'Publish native packages', link: '/advanced/publish-pypi' }
      ]
    }
  ],
  '/development/': [
    {
      text: 'Development',
      items: [
        { text: 'Local Desktop Development', link: '/development/local-desktop-build' },
        { text: 'Repository Structure', link: '/development/repository-structure' }
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
        { text: '安装', link: '/zh/guide/cli-2x#standalone-binary-download' },
        { text: '2.x CLI 下载', link: '/zh/guide/cli-2x' },
        { text: '使用 Agent Skills', link: '/zh/guide/using-agent-skills' },
        { text: '研究工作流', link: '/zh/guide/task-recipes' },
        { text: '多 Agent 运行', link: '/zh/guide/multi-agent' },
        { text: '升级', link: '/zh/guide/cli-2x#install-and-upgrade-bundled-content' },
        { text: '数据所有权与生命周期', link: '/zh/guide/data-lifecycle' },
        { text: '故障排除', link: '/zh/guide/troubleshooting' }
      ]
    }
  ],
  '/zh/reference/': [
    {
      text: '参考',
      items: [
        { text: '总览', link: '/zh/reference/' },
        { text: '2.x CLI', link: '/zh/guide/cli-2x' },
        { text: '1.x CLI（历史）', link: '/zh/reference/cli' },
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
        { text: '论文类型示例', link: '/zh/examples/paper-type-playbooks' },
        { text: 'Research Graph 完整示例', link: '/zh/examples/research-graph' }
      ]
    }
  ],
  '/zh/advanced/': [
    {
      text: '高级',
      items: [
        { text: '总览', link: '/zh/advanced/' },
        { text: '扩展 Qiongli', link: '/zh/advanced/extend-qiongli' },
        { text: '学科指导与内容包', link: '/zh/advanced/subject-packaging-model' },
        { text: '代理协作与 Hook', link: '/zh/advanced/agent-skill-collaboration' },
        { text: 'Plugin 如何接入', link: '/zh/advanced/plugin-first-architecture' },
        { text: '文献服务配置', link: '/zh/advanced/mcp-providers-setup' },
        { text: '严谨文献检索', link: '/zh/advanced/rigorous-literature-search' },
        { text: 'Zotero 集成', link: '/zh/advanced/mcp-zotero-integration' },
        { text: '发布原生渠道包', link: '/zh/advanced/publish-pypi' }
      ]
    }
  ],
  '/zh/maintainer/': [
    {
      text: '维护者',
      items: [
        { text: '总览', link: '/zh/maintainer/' },
        { text: '维护工作流程', link: '/zh/maintainer/claude-overview' },
        { text: '系统架构', link: '/zh/architecture' },
        { text: '规范约定', link: '/zh/conventions' },
        { text: '本地桌面开发', link: '/zh/development/local-desktop-build' },
        { text: '仓库结构', link: '/zh/development/repository-structure' },
        { text: '命名策略', link: '/zh/maintainer/naming-policy' },
        { text: '发布分支策略', link: '/zh/maintainer/release-branch-policy' },
        { text: '发布原生渠道包', link: '/zh/advanced/publish-pypi' }
      ]
    }
  ],
  '/zh/development/': [
    {
      text: '开发指南',
      items: [
        { text: '本地桌面开发', link: '/zh/development/local-desktop-build' },
        { text: '仓库结构', link: '/zh/development/repository-structure' }
      ]
    }
  ]
}

const localSearch = {
  provider: 'local',
  options: {
    _render(src, env, md) {
      if (/^(?:zh\/)?(?:superpowers|architecture\/decisions|archive|audits)\//.test(env.relativePath)) return ''
      if (/^(?:zh\/)?(?:development\/(?:ctr-|repository-restructuring)|maintainer\/(?:skill-quality-|skill-set-))/.test(env.relativePath)) return ''
      const html = md.render(src, env)
      return env.frontmatter?.search === false ? '' : html
    }
  }
}

const commonHead = [
  ['meta', { name: 'theme-color', content: '#0f766e' }],
  ['meta', { name: 'author', content: 'Jiaxin Peng' }]
]

/** @type {import('vitepress').UserConfig} */
export default {
  title: 'Qiongli',
  description: 'Use AI agents for academic research without losing the evidence trail.',
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
        search: localSearch,
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
        search: localSearch,
        outline: { level: [2, 3] },
        socialLinks: [{ icon: 'github', link: 'https://github.com/jxpeng98/qiongli' }],
        footer: {
          message: 'Qiongli 文档站',
          copyright: 'MIT License'
        }
      }
    }
  },
  themeConfig: {}
}
