// 🚀 AFNS-SPECIFIC FLUTTER WIDGET COLLECTION
// Direct AFNS Language Support in Flutter Widgets

library afns_widgets;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'dart:convert';
import '../dart_runtime/afns_runtime.dart';

// 🎯 MAIN AFNS WIDGET COLLECTION

class AFNSButton extends StatelessWidget {
  final String afnsCode;
  final VoidCallback? onPressed;
  final String text;
  final IconData? icon;
  final AFNSButtonStyle style;
  final EdgeInsetsGeometry? padding;

  const AFNSButton({
    Key? key,
    required this.afnsCode,
    this.onPressed,
    this.text = 'AFNS Action',
    this.icon,
    this.style = AFNSButtonStyle.primary,
    this.padding,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: padding,
      child: ElevatedButton.icon(
        onPressed: () {
          final result = AFNSRuntime.executeAFNSLogic(afnsCode);
          _showResult(context, result);
          onPressed?.call();
        },
        icon: Icon(icon ?? Icons.code),
        label: Text(text),
        style: _getButtonStyle(style),
      ),
    );
  }

  ButtonStyle _getButtonStyle(AFNSButtonStyle style) {
    switch (style) {
      case AFNSButtonStyle.primary:
        return ElevatedButton.styleFrom(
          backgroundColor: Colors.blue[800],
          foregroundColor: Colors.white,
        );
      case AFNSButtonStyle.success:
        return ElevatedButton.styleFrom(
          backgroundColor: Colors.green[600],
          foregroundColor: Colors.white,
        );
      case AFNSButtonStyle.warning:
        return ElevatedButton.styleFrom(
          backgroundColor: Colors.orange[600],
          foregroundColor: Colors.white,
        );
      case AFNSButtonStyle.danger:
        return ElevatedButton.styleFrom(
          backgroundColor: Colors.red[600],
          foregroundColor: Colors.white,
        );
      case AFNSButtonStyle.custom:
        return ElevatedButton.styleFrom(
          backgroundColor: Colors.purple[600],
          foregroundColor: Colors.white,
        );
    }
  }

  void _showResult(BuildContext context, dynamic result) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text('AFNS Result: $result'),
        duration: const Duration(seconds: 3),
      ),
    );
  }
}

enum AFNSButtonStyle { primary, success, warning, danger, custom }

class AFNSTextField extends StatefulWidget {
  final String afnsCode;
  final String label;
  final String hintText;
  final String? Function(String)? validator;
  final Function(String)? onChanged;
  final TextInputType keyboardType;

  const AFNSTextField({
    Key? key,
    required this.afnsCode,
    required this.label,
    this.hintText = 'Enter AFNS expression...',
    this.validator,
    this.onChanged,
    this.keyboardType = TextInputType.text,
  }) : super(key: key);

  @override
  State<AFNSTextField> createState() => _AFNSTextFieldState();
}

class _AFNSTextFieldState extends State<AFNSTextField> {
  late TextEditingController _controller;

  @override
  void initState() {
    super.initState();
    _controller = TextEditingController();
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          widget.label,
          style: const TextStyle(
            fontSize: 16,
            fontWeight: FontWeight.bold,
          ),
        ),
        const SizedBox(height: 8),
        TextFormField(
          controller: _controller,
          decoration: InputDecoration(
            hintText: widget.hintText,
            border: const OutlineInputBorder(),
            prefixIcon: const Icon(Icons.code),
            suffixIcon: IconButton(
              icon: const Icon(Icons.play_arrow),
              onPressed: () {
                final codeWithInput = widget.afnsCode.replaceAll('__INPUT__', _controller.text);
                final result = AFNSRuntime.executeAFNSLogic(codeWithInput);
                _showResult(context, result);
              },
            ),
          ),
          keyboardType: widget.keyboardType,
          validator: widget.validator,
          onChanged: widget.onChanged,
        ),
      ],
    );
  }

  void _showResult(BuildContext context, dynamic result) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text('AFNS Execution: $result'),
        duration: const Duration(seconds: 3),
      ),
    );
  }
}

class AFNSCard extends StatelessWidget {
  final String afnsCode;
  final String title;
  final String? subtitle;
  final IconData? icon;
  final Color? color;
  final Widget? child;

  const AFNSCard({
    Key? key,
    required this.afnsCode,
    required this.title,
    this.subtitle,
    this.icon,
    this.color,
    this.child,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Card(
      elevation: 4,
      child: InkWell(
        onTap: () {
          final result = AFNSRuntime.executeAFNSLogic(afnsCode);
          _showCardResult(context, result);
        },
        borderRadius: BorderRadius.circular(12),
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  if (icon != null) ...[
                    Icon(icon, color: color ?? Colors.blue),
                    const SizedBox(width: 12),
                  ],
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          title,
                          style: const TextStyle(
                            fontSize: 18,
                            fontWeight: FontWeight.bold,
                          ),
                        ),
                        if (subtitle != null) ...[
                          const SizedBox(height: 4),
                          Text(
                            subtitle!,
                            style: TextStyle(
                              color: Colors.grey[600],
                              fontSize: 14,
                            ),
                          ),
                        ],
                      ],
                    ),
                  ),
                  Icon(
                    Icons.touch_app,
                    color: Colors.grey[400],
                    size: 20,
                  ),
                ],
              ),
              if (child != null) ...[
                const SizedBox(height: 12),
                child!,
              ],
            ],
          ),
        ),
      ),
    );
  }

  void _showCardResult(BuildContext context, dynamic result) {
    showDialog(
      context: context,
      builder: (BuildContext context) {
        return AlertDialog(
          title: Text('AFNS Result: $title'),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Container(
                padding: const EdgeInsets.all(12),
                decoration: BoxDecoration(
                  color: Colors.grey[100],
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Text(
                  result.toString(),
                  style: const TextStyle(fontFamily: 'monospace'),
                ),
              ),
              const SizedBox(height: 12),
              Container(
 lement: Column(
                children: [
                  Text('AFNS Code:'),
                  Text(
                    afnsCode,
                    style: const TextStyle(
                      fontFamily: 'monospace',
                      fontSize: 12,
                    ),
                  ),
                ],
              ),
            ],
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('OK'),
            ),
          ],
        );
      },
    );
  }
}

class AFNSListTile extends StatelessWidget {
  final String afnsCode;
  final String title;
  final String? subtitle;
  final IconData? leadingIcon;
  final IconData? trailingIcon;
  final Function()? onTap;

  const AFNSListTile({
    Key? key,
    required this.afnsCode,
    required this.title,
    this.subtitle,
    this.leadingIcon,
    this.trailingIcon,
    this.onTap,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return ListTile(
      leading: Icon(leadingIcon ?? Icons.code),
      title: Text(title),
      subtitle: subtitle != null ? Text(subtitle!) : null,
      trailing: Icon(trailingIcon ?? Icons.play_arrow),
      onTap: () {
        final result = AFNSRuntime.executeAFNSLogic(afnsCode);
        _showTileResult(context, result);
        onTap?.call();
      },
    );
  }

  void _showTileResult(BuildContext context, dynamic result) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text('AFNS Execution: $result'),
        backgroundColor: Colors.blue[600],
      ),
    );
  }
}

class AFNSFloatingActionButton extends StatelessWidget {
  final String afnsCode;
  final IconData icon;
  final String? tooltip;
  final Function()? onPressed;

  const AFNSFloatingActionButton({
    Key? key,
    required this.afnsCode,
    this.icon = Icons.code,
    this.tooltip,
    this.onPressed,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return FloatingActionButton(
      onPressed: () {
        final result = AFNSRuntime.executeAFNSLogic(afnsCode);
        _showFABResult(context, result);
        onPressed?.call();
      },
      tooltip: tooltip ?? 'Execute AFNS Code',
      child: Icon(icon),
      backgroundColor: Colors.purple[600],
    );
  }

  void _showFABResult(BuildContext context, dynamic result) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text('AFNS Result: $result'),
        backgroundColor: Colors.purple[600],
        duration: const Duration(seconds: 4),
      ),
    );
  }
}

class AFNSContainer extends StatelessWidget {
  final String afnsCode;
  final Widget child;
  final EdgeInsetsGeometry? padding;
  final EdgeInsetsGeometry? margin;
  final Color? color;
  final Decoration? decoration;
  final double? width;
  final double? height;

  const AFNSContainer({
    Key? key,
    required this.afnsCode,
    required this.child,
    this.padding,
    this.margin,
    this.color,
    this.decoration,
    this.width,
    this.height,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: () {
        final result = AFNSRuntime.executeAFNSLogic(afnsCode);
        _showContainerResult(context, result);
      },
      child: Container(
        width: width,
        height: height,
        padding: padding,
        margin: margin,
        color: color,
        decoration: decoration,
        child: child,
      ),
    );
  }

  void _showContainerResult(BuildContext context, dynamic result) {
    showDialog(
      context: context,
      builder: (BuildContext context) {
        return AlertDialog(
          title: const Text('AFNS Container Result'),
          content: Text(result.toString()),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('OK'),
            ),
          ],
        );
      },
    );
  }
}

class AFNSAppBar extends StatelessWidget implements PreferredSizeWidget {
  final String afnsCode;
  final String title;
  final List<Widget>? actions;
  final Widget? leading;
  final Color? backgroundColor;
  final Color? foregroundColor;

  const AFNSAppBar({
    Key? key,
    required this.afnsCode,
    required this.title,
    this.actions,
    this.leading,
    this.backgroundColor,
    this.foregroundColor,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return AppBar(
      title: Text(title),
      backgroundColor: backgroundColor,
      foregroundColor: foregroundColor,
      leading: leading,
      actions: [
        if (actions != null) ...actions,
        IconButton(
          icon: const Icon(Icons.code),
          onPressed: () {
            final result = AFNSRuntime.executeAFNSLogic(afnsCode);
            _showAppBarResult(context, result);
          },
          tooltip: 'Execute AFNS Code',
        ),
      ],
    );
  }

  @override
  Size get preferredSize => const Size.fromHeight(kToolbarHeight);

  void _showAppBarResult(BuildContext context, dynamic result) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text('AppBar AFNS: $result'),
        backgroundColor: Colors.indigo[600],
      ),
    );
  }
}

class AFNSBottomNavigationBar extends StatelessWidget {
  final String afnsCode;
  final int currentIndex;
  final Function(int) onTap;
  final List<AFNSBottomNavigationTab> tabs;

  const AFNSBottomNavigationBar({
    Key? key,
    required this.afnsCode,
    required this.currentIndex,
    required this.onTap,
    required this.tabs,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return BottomNavigationBar(
      currentIndex: currentIndex,
      onTap: (index) {
        final result = AFNSRuntime.executeAFNSLogic(afnsCode);
        _showBottomNavResult(context, result);
        onTap(index);
      },
      items: tabs.map((tab) => BottomNavigationBarItem(
        icon: Icon(tab.icon),
        label: tab.label,
        backgroundColor: tab.backgroundColor,
      )).toList(),
    );
  }

  void _showBottomNavResult(BuildContext context, dynamic result) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text('Bottom Nav AFNS: $result'),
        backgroundColor: Colors.teal[600],
      ),
    );
  }
}

class AFNSBottomNavigationTab {
  final String label;
  final IconData icon;
  final Color? backgroundColor;

  const AFNSBottomNavigationTab({
    required this.label,
    required this.icon,
    this.backgroundColor,
  });
}

// 🎯 AFNS WIDGET UTILITIES

class AFNSWidgetUtils {
  // Create multiple AFNS buttons in a row
  static Widget createAFNSButtonRow(List<AFNSButtonConfig> configs) {
    return Row(
      children: configs.map((config) => Expanded(
        child: AFNSButton(
          afnsCode: config.code,
          text: config.text,
          icon: config.icon,
          style: config.style,
        ),
      )).toList(),
    );
  }

  // Create AFNS form with validation
  static Widget createAFNSForm({
    required List<AFNSFormField> fields,
    required afnsCode,
    required GlobalKey<FormState> formKey,
    required Function(Map<String, String>) onSubmit,
  }) {
    final textControllers = <String, TextEditingController>{};
    
    return Form(
      key: formKey,
      child: Column(
        children: [
          ...fields.map((field) => AFNSTextField(
            afnsCode: field.validationCode,
            label: field.label,
            hintText: field.hintText,
            keyboardType: field.keyboardType,
          )),
          const SizedBox(height: 16),
          AFNSButton(
            afnsCode: afnsCode,
            text: 'Submit Form',
            icon: Icons.check,
            style: AFNSButtonStyle.success,
          ),
        ],
      ),
    );
  }
}

class AFNSButtonConfig {
  final String code;
  final String text;
  final IconData? icon;
  final AFNSButtonStyle style;

  const AFNSButtonConfig({
    required this.code,
    required this.text,
    this.icon,
    this.style = AFNSButtonStyle.primary,
  });
}

class AFNSFormField {
  final String validationCode;
  final String label;
  final String hintText;
  final TextInputType keyboardType;

  const AFNSFormField({
    required this.validationCode,
    required this.label,
    this.hintText = 'Enter value...',
    this.keyboardType = TextInputType.text,
  });
}
