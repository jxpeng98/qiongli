// The two languages share categories and routes; labels are written for each reader.
const categories = [
  {
    en: 'Getting started', zh: '入门', route: '/guide/',
    items: [
      ['Overview', '入门概览', '/guide/'],
      ['Quickstart', '快速开始', '/quickstart'],
      ['Installation', '安装', '/guide/install'],
      ['Upgrade and rollback', '升级与回退', '/guide/upgrade'],
      ['What changed in 2.x', '2.x 变化', '/guide/whats-new-2'],
      ['Troubleshooting', '故障排除', '/guide/troubleshooting']
    ]
  },
  {
    en: 'Research', zh: '研究', route: '/guide/task-recipes',
    items: [
      ['Choose a task', '选择研究任务', '/guide/task-recipes'],
      ['Using Skills', '使用 Skills', '/guide/using-agent-skills'],
      ['Literature search', '文献检索', '/advanced/rigorous-literature-search'],
      ['Work with other agents', '与其他代理协作', '/guide/multi-agent'],
      ['Examples', '示例', '/examples/'],
      ['Routes by paper type', '论文类型路线', '/examples/paper-type-playbooks'],
      ['Research Graph', 'Research Graph 示例', '/examples/research-graph'],
      ['Data and backups', '数据与备份', '/guide/data-lifecycle']
    ]
  },
  {
    en: 'Connections', zh: '接入', route: '/advanced/',
    items: [
      ['Overview', '接入与配置', '/advanced/'],
      ['Plugin setup', 'Plugin 配置', '/advanced/plugin-installation'],
      ['Plugin contents', 'Plugin 中的内容', '/advanced/plugin-first-architecture'],
      ['MCP connections', 'MCP 接入', '/advanced/cross-platform-mcp'],
      ['Literature providers', '文献服务', '/advanced/mcp-providers-setup'],
      ['Zotero', 'Zotero', '/advanced/mcp-zotero-integration'],
      ['Collaboration and hooks', '协作与 Hook', '/advanced/agent-skill-collaboration'],
      ['External Agent coordination', '外部 Agent 协作', '/advanced/external-host-coordination'],
      ['Finance and economics data', '金融与经济学数据', '/advanced/finance-econ-data-mcp']
    ]
  },
  {
    en: 'Reference', zh: '参考', route: '/reference/',
    items: [
      ['Overview', '参考概览', '/reference/'],
      ['CLI commands', 'CLI 命令', '/guide/cli-2x'],
      ['Skills guide', 'Skills 指南', '/reference/skills'],
      ['Host capabilities', 'Host 能力', '/guide/agent-host-capability-matrix'],
      ['Architecture', '系统架构', '/architecture'],
      ['Editing conventions', '编辑约定', '/conventions']
    ]
  },
  {
    en: 'Development', zh: '开发', route: '/development/',
    items: [
      ['Development and maintenance', '开发与维护', '/development/'],
      ['Repository structure', '仓库结构', '/development/repository-structure'],
      ['Extend Qiongli', '扩展穷理', '/advanced/extend-qiongli'],
      ['Subject guidance', '学科指导', '/advanced/subject-packaging-model'],
      ['Maintainer guide', '维护指南', '/maintainer/'],
      ['Maintainer workflow', '维护流程', '/maintainer/claude-overview'],
      ['Naming rules', '命名规则', '/maintainer/naming-policy'],
      ['Adapt external ideas', '借鉴外部方法', '/maintainer/external-borrowing'],
      ['Release branch policy', '发布分支规则', '/maintainer/release-branch-policy'],
      ['Publish native packages', '发布原生包', '/advanced/publish-pypi']
    ]
  },
  {
    en: 'Historical material', zh: '历史资料', route: '/legacy/',
    items: [
      ['Historical material and records', '历史资料与项目记录', '/legacy/'],
      ['1.x installation', '1.x 安装', '/legacy/install'],
      ['1.x upgrades', '1.x 升级', '/legacy/upgrade'],
      ['1.x CLI reference', '1.x 命令参考', '/reference/cli'],
      ['Retained Desktop packages', '保留的桌面安装包', '/advanced/native-desktop-alpha'],
      ['Local Desktop development', '本地桌面开发', '/development/local-desktop-build']
    ]
  }
]

function navigation(language) {
  const prefix = language === 'zh' ? '/zh' : ''
  return categories.map(category => ({ text: category[language], link: prefix + category.route }))
}

function sidebar(language) {
  const prefix = language === 'zh' ? '/zh' : ''
  return {
    [prefix + '/']: categories.map(category => ({
      text: category[language],
      collapsed: true,
      items: category.items.map(([en, zh, route]) => ({
        text: language === 'zh' ? zh : en,
        link: prefix + route
      }))
    }))
  }
}

const enNav = navigation('en')
const zhNav = navigation('zh')
const enSidebar = sidebar('en')
const zhSidebar = sidebar('zh')

const localSearch = {
  provider: 'local',
  options: {
    _render(src, env, md) {
      if (/^(?:zh\/)?(?:superpowers|architecture\/decisions|legacy|archive|audits)\//.test(env.relativePath)) return ''
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
  description: 'Read, design and write with AI agents, with sources you can check.',
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
