# Initial Concept
An AdGuard Home MCP server written in Rust to allow AI models to interact with and manage AdGuard Home instances.

# Product Definition

## Target Audience
- **Home Users:** Individuals seeking to manage their network security and filtering through natural language interfaces and AI assistants.

## Goals
- **Seamless Monitoring:** Provide a simple, intuitive interface for AI models to query real-time AdGuard Home statistics, such as blocked queries and top clients.
- **Intelligent Control:** Enable AI-driven management of filtering rules, protection toggles, and specific client settings.
- **Deep Insights:** Offer sophisticated log analysis and real-time notifications via the Model Context Protocol, allowing for proactive network management.

## Key Features
- **Core Protection Management:** Tools to instantly toggle global protection and specific safety features like Safe Search, Safe Browsing, and Parental Control.
- **Activity Monitoring & Stats:** Tools to retrieve global network statistics and search through DNS query logs for detailed activity analysis.
- **Token-Optimized Lazy Loading:** "Lazy Mode" to reduce context usage by loading tools on demand.
- **Network Accessibility:** Support for HTTP/SSE transport, allowing connection over the network with Bearer Token authentication.
- **Log Intelligence (RAG):** Capabilities for retrieval-augmented generation to allow AI models to analyze historical logs and identify trends or anomalies.
- **Smart Client Management:** Automated discovery and configuration of network clients using AI-driven commands.
- **Dynamic DNS Rewrites:** Tools to list, add, and remove DNS rewrite entries, enabling AI-driven network-wide domain redirections.
- Blocklist Management: Comprehensive management of filtering lists, allowing AI models to audit, toggle, and add community-driven blocklists and allowlists.
- Network Device Inventory: Tools to list and inspect network clients, providing AI models with a complete view of devices and their individual protection status.
- Filtering Rule Debugger: Tools to check why a specific domain is blocked or allowed, identifying the exact rule responsible.
- System Maintenance: Tools to clear historical statistics and query logs for privacy and data management.
- Query Log Management: Tools to configure DNS query log retention, anonymization, and client-specific logging settings.
- Update & System Info: Tools to retrieve version information, check for available updates, and trigger AdGuard Home updates.
