// 🚀 COMPLETE AFNS GUI APPLICATION
// ApexForge NightScript Direct Flutter Integration Demo

import 'package:flutter/material.dart';
import '../dart_runtime/afns_runtime.dart';
import '../widgets/afns_widgets.dart';

void main() async {
  // Initialize AFNS Runtime
  await AFNSRuntime.initialize();
  
  runApp(const AFNSGUIApp());
}

class AFNSGUIApp extends StatelessWidget {
  const AFNSGUIApp({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: '🎯 AFNS Direct Flutter Integration',
      theme: ThemeData(
        primarySwatch: Colors.blue,
        useMaterial3: true,
        visualDensity: VisualDensity.adaptivePlatformDensity,
      ),
      home: const AFNSHomePage(),
      debugShowCheckedModeBanner: false,
    );
  }
}

class AFNSHomePage extends StatefulWidget {
  const AFNSHomePage({Key? key}) : super(key: key);

  @override
  State<AFNSHomePage> createState() => _AFNSHomePageState();
}

class _AFNSHomePageState extends State<AFNSHomePage> {
  int _selectedIndex = 0;
  String _currentAFNSCode = 'fun welcome() -> string { return "Welcome to AFNS!"; }';
  String _runtimeStatus = 'AFNS Runtime Ready';
  
  // AFNS Business Logic Variables
  double _companyRevenue = 1000000.0;
  int _employees = 0;
  List<String> _employeeList = [];
  String _lastExecutionResult = '';

  final List<Widget> _pages = [];

  @override
  void initState() {
    super.initState();
    _initializePages();
    _updateRuntimeStatus();
  }

  void _initializePages() {
    _pages.addAll([
      AFNSDashboardPage(
        companyRevenue: _companyRevenue,
        employees: _employees,
        employeeList: _employeeList,
        lastResult: _lastExecutionResult,
        onRevenueUpdate: (revenue) {
          setState(() {
            _companyRevenue = revenue;
          });
        },
        onEmployeeUpdate: (employees, empList) {
          setState(() {
            _employees = employees;
            _employeeList = empList;
          });
        },
        onResultUpdate: (result) {
          setState(() {
            _lastExecutionResult = result;
          });
        },
      ),
      AFNSCodeEditorPage(
        afnsCode: _currentAFNSCode,
        onCodeChange: (code) {
          setState(() {
            _currentAFNSCode = code;
          });
        },
        onExecute: (result) {
          setState(() {
            _lastExecutionResult = result;
          });
        },
      ),
      AFNSBusinessPage(),
      AFNSWidgetGalleryPage(),
    ]);
  }

  void _updateRuntimeStatus() {
    _runtimeStatus = AFNSRuntime.getAFNSState();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AFNSAppBar(
        afnsCode: 'fun appBarAction() { show("AFNS AppBar Activated!"); }',
        title: '🎯 ApexForge NightScript GUI',
        backgroundColor: Colors.blue[800],
        foregroundColor: Colors.white,
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: () {
              setState(() {
                _updateRuntimeStatus();
              });
            },
          ),
          IconButton(
            icon: const Icon(Icons.info),
            onPressed: () {
              showDialog(
                context: context,
                builder: (context) => AlertDialog(
                  title: const Text('🚀 AFNS Direct Integration'),
                  content: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Text('Runtime Status: $_runtimeStatus'),
                      const SizedBox(height: 8),
                      const Text('Company Revenue: \$${_companyRevenue.toStringAsFixed(0)}'),
                      Text('Employees: $_employees'),
                      Text('Last Result: $_lastExecutionResult'),
                    ],
                  ),
                  actions: [
                    TextButton(
                      onPressed: () => Navigator.pop(context),
                      child: const Text('OK'),
                    ),
                  ],
                ),
              );
            },
          ),
        ],
      ),
      body: IndexedStack(
        index: _selectedIndex,
        children: _pages,
      ),
      bottomNavigationBar: AFNSBottomNavigationBar(
        afnsCode: 'fun switchPage(page::i32) { show("Switching to page " + page); }',
        currentIndex: _selectedIndex,
        onTap: (index) {
          setState(() {
            _selectedIndex = index;
          });
        },
        tabs: const [
          AFNSBottomNavigationTab(
            label: 'Dashboard',
            icon: Icons.dashboard,
          ),
          AFNSBottomNavigationTab(
            label: 'Code Editor',
            icon: Icons.code,
          ),
          AFNSBottomNavigationTab(
            label: 'Business',
            icon: Icons.business_center,
          ),
          AFNSBottomNavigationTab(
            label: 'Widgets',
            icon: Icons.widgets,
          ),
        ],
      ),
      floatingActionButton: AFNSFloatingActionButton(
        afnsCode: 'fun floatingAction() { show("🎯 AFNS Floating Action!"); }',
        icon: Icons.rocket_launch,
        tooltip: 'Execute AFNS Floating Action',
      ),
    );
  }
}

// 🎯 DASHBOARD PAGE
class AFNSDashboardPage extends StatelessWidget {
  final double companyRevenue;
  final int employees;
  final List<String> employeeList;
  final String lastResult;
  final Function(double) onRevenueUpdate;
  final Function(int, List<String>) onEmployeeUpdate;
  final Function(String) onResultUpdate;

  const AFNSDashboardPage({
    Key? key,
    required this.companyRevenue,
    required this.employees,
    required this.employeeList,
    required this.lastResult,
    required this.onRevenueUpdate,
    required that.onEmployeeUpdate,
    required this.onResultUpdate,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // AFNS Welcome Card
          AFNSCard(
            afnsCode: 'fun welcomeCompany() { show("Company Dashboard Ready!"); }',
            title: '🏢 Company Dashboard',
            subtitle: 'AFNS-Powered Business Management',
            icon: Icons.business,
            color: Colors.blue,
            child: Column(
              children: [
                const Text('Revenue Growth Simulation'),
                LinearProgressIndicator(
                  value: (companyRevenue / 10000000).clamp(0.0, 1.0),
                  backgroundColor: Colors.grey[300],
                  valueColor: AlwaysStoppedAnimation<Color>(Colors.green[600]!),
                ),
              ],
            ),
          ),
          
          const SizedBox(height: 16),
          
          // Revenue Actions Row
          AFNSWidgetUtils.createAFNSButtonRow([
            AFNSButtonConfig(
              code: '''fun growRevenue() {
                var newRevenue::f64 = $companyRevenue * 1.20; // 20% growth
                show("Revenue: $" + newRevenue);
                return "Revenue Updated!";
              }''',
              text: '💰 Grow Revenue',
              icon: Icons.trending_up,
              style: AFNSButtonStyle.success,
            ),
            AFNSButtonConfig(
              code: '''fun optimizeBusiness() {
                var optimized::f64 = $companyRevenue * 1.15; // 15% optimization
                show("Optimized: $" + optimized);
                return "Business Optimized!";
              }''',
              text: '🎯 Optimize',
              icon: Icons.tune,
              style: AFNSButtonStyle.primary,
            ),
          ]),
          
          const SizedBox(height: 16),
          
          // Employee Management
          Row(
            children: [
              Expanded(
                child: AFNSContainer(
                  afnsCode: '''fun showRevenue() {
                    show("Current Revenue: $companyRevenue");
                    return "$companyRevenue";
                  }''',
                  child: AFNSWidgetUtils.createAFNSDisplay(
                    title: 'Revenue',
                    content: '\$${companyRevenue.toStringAsFixed(0)}',
                    icon: Icons.attach_money,
                    color: Colors.green,
                  ),
                ),
              ),
              const SizedBox(width: 16),
              Expanded(
                child: AFNSContainer(
                  afnsCode: '''fun showEmployees() {
                    show("Current Employees: $employees");
                    return "$employees";
                  }''',
                  child: AFNSWidgetUtils.createAFNSDisplay(
                    title: 'Employees',
                    content: employees.toString(),
                    icon: Icons.people,
                    color: Colors.blue,
                  ),
                ),
              ),
            ],
          ),
          
          const SizedBox(height: 16),
          
          // Employee Actions
          AFNSWidgetUtils.createAFNSButtonRow([
            AFNSButtonConfig(
              code: '''fun hireEmployee() {
                var hireCount::i32 = $employees + 1;
                show("Hired new employee! Total: " + hireCount);
                return "Employee Hired!";
              }''',
              text: '👥 Hire',
              icon: Icons.person_add,
              style: AFNSButtonStyle.success,
            ),
            AFNSButtonConfig(
              code: '''fun fireEmployee() {
                var fireCount::i32 = $employees - 1;
                if fireCount >= 0 {
                  show("Employee released! Remaining: " + fireCount);
                  return "Employee Released";
                } else {
                  show("No employees to release!");
                  return "Empty Staff";
                }
              }''',
              text: '❌ Release',
              icon: Icons.person_remove,
              style: AFNSButtonStyle.danger,
            ),
          ]),
          
          const SizedBox(height: 16),
          
          // Last Execution Result Card
          if (lastResult.isNotEmpty)
            AFNSCard(
              afnsCode: 'fun showLastResult() { show("$_lastExecutionResult"); return "Result Displayed"; }',
              title: '📊 Last Execution Result',
              subtitle: 'AFNS Latest Output',
              icon: Icons.output,
              color: Colors.purple,
              child: Container(
                padding: const EdgeInsets.all(12),
                decoration: BoxDecoration(
                  color: Colors.purple[50],
                  borderRadius: BorderRadius.circular(8),
                  border: Border.all(color: Colors.purple[200]!),
                ),
                child: Text(
                  lastResult,
                ),
              ),
            ),
        ],
      ),
    );
  }
}

// Code Editor Page
class AFNSCodeEditorPage extends StatefulWidget {
  final String afnsCode;
  final Function(String) onCodeChange;
  final Function(String) onExecute;

  const AFNSCodeEditorPage({
    Key? key,
    required this.afnsCode,
    required this.onCodeChange,
    required this.onExecute,
  }) : super(key: key);

  @override
  State<AFNSCodeEditorPage> createState() => _AFNSCodeEditorPageState();
}

class _AFNSCodeEditorPageState extends State<AFNSCodeEditorPage> {
  late TextEditingController _controller;

  @override
  void initState() {
    super.initState();
    _controller = TextEditingController(text: widget.afnsCode);
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        children: [
          AFNSCard(
            afnsCode: 'fun editorAction() { show("Code Editor Ready!"); return "Editor Active"; }',
            title: '🎯 AFNS Code Editor',
            subtitle: 'Direct AFNS Code Execution',
            icon: Icons.code,
            color: Colors.teal,
          ),
          
          const SizedBox(height: 16),
          
          Expanded(
            child: AFNSContainer(
              afnsCode: 'fun codeContainer() { show("Code container activated!"); return "Container Active"; }',
              decoration: BoxDecoration(
                border: Border.all(color: Colors.grey[300]!),
                borderRadius: BorderRadius.circular(8),
              ),
              child: Padding(
                padding: const EdgeInsets.all(8),
                child: TextField(
                  controller: _controller,
                  maxLines: null,
                  expands: true,
                  decoration: const InputDecoration(
                    hintText: 'Enter your AFNS code here...\n\nExample:\nfun myFunction() -> string {\n  return "Hello from AFNS!";\n}\n\napex() {\n  show(myFunction());\n}',
                    border: InputBorder.none,
                  ),
                  style: const TextStyle(
                    fontFamily: 'monospace',
                    fontSize: 14,
                  ),
                  onChanged: (value) {
                    widget.onCodeChange(value);
                  },
                ),
              ),
            ),
          ),
          
          const SizedBox(height: 16),
          
          AFNSButton(
            afnsCode: widget.afnsCode,
            text: '🚀 Execute AFNS Code',
            icon: Icons.play_arrow,
            style: AFNSButtonStyle.success,
            onPressed: () {
              final result = AFNSRuntime.executeAFNSLogic(widget.afnsCode);
              widget.onExecute(result.toString());
            },
          ),
        ],
      ),
    );
  }
}

// Business Page
class AFNSBusinessPage extends StatefulWidget {
  @override
  State<AFNSBusinessPage> createState() => _AFNSBusinessPageState();
}

class _AFNSBusinessPageState extends State<AFNSBusinessPage> {
  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        children: [
          AFNSCard(
            afnsCode: 'fun businessCard() { show("Business Operations!"); return "Business Active"; }',
            title: '🏢 AFNS Business Operations',
            icon: Icons.business_center,
            color: Colors.indigo,
          ),
          
          const SizedBox(height: 16),
          
          // Business Operations Grid
          GridView.count(
            shrinkWrap: true,
            physics: const NeverScrollableScrollPhysics(),
            crossAxisCount: 2,
            crossAxisSpacing: 16,
            mainAxisSpacing: 16,
            children: [
              AFNSCard(
                afnsCode: 'fun salesOperation() { show("Sales Pipeline Activated!"); return "Sales Active"; }',
                title: '📈 Sales',
                icon: Icons.trending_up,
                color: Colors.green,
                child: const Text('Revenue Management'),
              ),
              
              AFNSCard(
                afnsCode: 'fun marketingOperation() { show("Marketing Campaign Started!"); return "Marketing Active"; }',
                title: '📢 Marketing',
                icon: Icons.campaign,
                color: Colors.blue,
                child: const Text('Campaign Management'),
              ),
              
              AFNSCard(
                afnsCode: 'fun hrOperation() { show("HR System Activated!"); return "HR Active"; }',
                title: '👥 Human Resources',
                icon: Icons.people,
                color: Colors.orange,
                child: const Text('Staff Management'),
              ),
              
              AFNSCard(
                afnsCode: 'fun financeOperation() { show("Financial Analysis Started!"); return "Finance Active"; }',
                title: '💰 Finance',
                icon: Icons.account_balance_wallet,
                color: Colors.red,
                child: const Text('Financial Planning'),
              ),
            ],
          ),
          
          const SizedBox(height: 16),
          
          AFNSButton(
            afnsCode: '''fun generateReport() {
              show("Generating comprehensive business report...");
              return "Business Report Generated!";
            }''',
            text: '📊 Generate Business Report',
            icon: Icons.assessment,
            style: AFNSButtonStyle.primary,
          ),
        ],
      ),
    );
  }
}

// Widget Gallery Page
class AFNSWidgetGalleryPage extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        children: [
          AFNSCard(
            afnsCode: 'fun widgetGallery() { show("Widget Gallery Activated!"); return "Gallery Active"; }',
            title: '🎨 AFNS Widget Gallery',
            icon: Icons.palette,
            color: Colors.pink,
          ),
          
          const SizedBox(height: 16),
          
          // Widget Examples
          AFNSTextField(
            afnsCode: '''fun validateInput(input::string) {
              if input.length > 0 {
                return "Valid: " + input;
              } else {
                return "Invalid input";
              }
            }''',
            label: 'AFNS Input Field',
            hintText: 'Enter text for validation...',
            keyboardType: TextInputType.text,
          ),
          
          const SizedBox(height: 16),
          
          // AFNS List Demo
          AFNSListTile(
            afnsCode: '''fun listAction() {
              show("List item selected!");
              return "List Action Executed";
            }''',
            title: 'AFNS List Item',
            subtitle: 'Tap to execute AFNS logic',
            leadingIcon: Icons.list,
            trailingIcon: Icons.arrow_forward,
          ),
          
          // More AFNS List Items
          AFNSListTile(
            afnsCode: '''fun businessMetrics() {
              show("Business metrics calculated!");
              return "Metrics Updated";
            }''',
            title: 'Business Metrics',
            subtitle: 'Revenue: \$1M, Profit: \$200K',
            leadingIcon: Icons.analytics,
          ),
          
          AFNSListTile(
            afnsCode: '''fun teamOverview() {
              show("Team overview generated!");
              return "Team Data Updated";
            }''',
            title: 'Team Overview',
            subtitle: '5 developers, 2 designers',
            leadingIcon: Icons.group,
          ),
          
          const SizedBox(height: 16),
          
          AFNSButton(
            afnsCode: '''fun refreshWidgets() {
              show("Widget gallery refreshed!");
              return "Gallery Updated";
            }''',
            text: '🔄 Refresh Widget Gallery',
            icon: Icons.refresh,
            style: AFNSButtonStyle.custom,
          ),
        ],
      ),
    );
  }
}
