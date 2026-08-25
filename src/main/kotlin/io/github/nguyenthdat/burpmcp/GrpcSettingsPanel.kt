package io.github.nguyenthdat.burpmcp

import burp.api.montoya.MontoyaApi
import burp.api.montoya.core.Registration
import burp.api.montoya.ui.settings.SettingsPanel
import java.awt.BorderLayout
import java.awt.Component
import java.awt.FlowLayout
import java.awt.GridBagConstraints
import java.awt.GridBagLayout
import java.awt.Insets
import java.nio.file.Path
import javax.swing.BorderFactory
import javax.swing.ButtonGroup
import javax.swing.JButton
import javax.swing.JLabel
import javax.swing.JOptionPane
import javax.swing.JPanel
import javax.swing.JRadioButton
import javax.swing.JSpinner
import javax.swing.JComponent
import javax.swing.JTextField
import javax.swing.SpinnerNumberModel
import javax.swing.SwingUtilities

internal class GrpcSettingsPanel(
    private val api: MontoyaApi,
    private val store: GrpcSettingsStore,
    private val lifecycle: TransportLifecycle,
    private val tlsBundles: TlsBundleManager = TlsBundleManager(),
) : SettingsPanel, AutoCloseable {
    private val root = JPanel(BorderLayout(0, 16))
    private val bindAddress = JTextField(28)
    private val port = JSpinner(SpinnerNumberModel(DEFAULT_GRPC_PORT, 1, 65535, 1))
    private val localPlaintext = JRadioButton("Local plaintext (127.0.0.1 only)")
    private val remoteMtls = JRadioButton("Remote mutual TLS")
    private val serverNames = JTextField(42)
    private val tlsDirectory = JTextField(42)
    private val status = JLabel()
    private val registration: Registration

    init {
        root.border = BorderFactory.createEmptyBorder(16, 16, 16, 16)
        root.add(form(), BorderLayout.NORTH)
        root.add(status, BorderLayout.SOUTH)
        api.userInterface().applyThemeToComponent(root)
        load(store.load())
        registration = api.userInterface().registerSettingsPanel(this)
    }

    override fun uiComponent(): JComponent = root

    override fun keywords(): Set<String> = setOf("Burp MCP", "gRPC", "mTLS", "certificate", "remote", "port")

    override fun close() = registration.deregister()

    private fun form(): JPanel {
        val panel = JPanel(GridBagLayout())
        val group = ButtonGroup().apply {
            add(localPlaintext)
            add(remoteMtls)
        }
        localPlaintext.addActionListener { updateEnabledState() }
        remoteMtls.addActionListener { updateEnabledState() }
        var row = 0
        addRow(panel, row++, "Bind address", bindAddress)
        addRow(panel, row++, "Port", port)
        addRow(panel, row++, "Security", JPanel().apply {
            layout = javax.swing.BoxLayout(this, javax.swing.BoxLayout.Y_AXIS)
            add(localPlaintext)
            add(remoteMtls)
        })
        addRow(panel, row++, "Certificate DNS names/IPs", serverNames)
        addRow(panel, row++, "TLS directory", tlsDirectory)
        addRow(panel, row, "Actions", JPanel(FlowLayout(FlowLayout.LEFT, 8, 0)).apply {
            add(JButton("Apply and restart server").apply { addActionListener { applyAndRestart() } })
            add(JButton("Rotate certificates").apply { addActionListener { rotateCertificates() } })
            add(JButton("Reset local defaults").apply { addActionListener { load(GrpcSettings()) } })
        })
        return panel
    }

    private fun addRow(panel: JPanel, row: Int, label: String, component: Component) {
        panel.add(JLabel(label), GridBagConstraints().apply {
            gridx = 0
            gridy = row
            anchor = GridBagConstraints.NORTHWEST
            insets = Insets(6, 0, 6, 16)
        })
        panel.add(component, GridBagConstraints().apply {
            gridx = 1
            gridy = row
            weightx = 1.0
            fill = GridBagConstraints.HORIZONTAL
            anchor = GridBagConstraints.NORTHWEST
            insets = Insets(6, 0, 6, 0)
        })
    }

    private fun readSettings(): GrpcSettings = GrpcSettings(
        bindAddress = bindAddress.text.trim(),
        port = (port.value as Number).toInt(),
        securityMode = if (remoteMtls.isSelected) GrpcSecurityMode.REMOTE_MTLS else GrpcSecurityMode.LOCAL_PLAINTEXT,
        serverNames = GrpcSettingsStore.parseServerNames(serverNames.text),
        tlsDirectory = Path.of(tlsDirectory.text.trim()).toAbsolutePath().normalize(),
    ).also(GrpcSettings::validate)

    private fun load(settings: GrpcSettings) {
        bindAddress.text = settings.bindAddress
        port.value = settings.port
        localPlaintext.isSelected = settings.securityMode == GrpcSecurityMode.LOCAL_PLAINTEXT
        remoteMtls.isSelected = settings.securityMode == GrpcSecurityMode.REMOTE_MTLS
        serverNames.text = settings.serverNames.joinToString(",")
        tlsDirectory.text = settings.tlsDirectory.toString()
        status.text = "Current server: ${lifecycle.settings()?.let { "${it.endpointScheme}://${it.bindAddress}:${it.port}" } ?: "stopped"}"
        updateEnabledState()
    }

    private fun updateEnabledState() {
        val remote = remoteMtls.isSelected
        serverNames.isEnabled = remote
        tlsDirectory.isEnabled = remote
        if (!remote) bindAddress.text = DEFAULT_GRPC_BIND_ADDRESS
    }

    private fun applyAndRestart() {
        runAction("gRPC server restarted") {
            val settings = readSettings()
            if (settings.securityMode == GrpcSecurityMode.REMOTE_MTLS) tlsBundles.ensure(settings)
            lifecycle.restart(settings)
            store.save(settings)
            load(settings)
        }
    }

    private fun rotateCertificates() {
        runAction("Certificates rotated; gRPC server restarted") {
            val settings = readSettings()
            require(settings.securityMode == GrpcSecurityMode.REMOTE_MTLS) { "Select Remote mutual TLS before rotating certificates" }
            tlsBundles.generate(settings.tlsDirectory, settings.serverNames)
            lifecycle.restart(settings)
            store.save(settings)
            load(settings)
        }
    }

    private fun runAction(success: String, action: () -> Unit) {
        try {
            action()
            status.text = success
        } catch (exception: Exception) {
            api.logging().logToError("[MCP] settings action failed", exception)
            status.text = "Error: ${exception.message}"
            JOptionPane.showMessageDialog(root, exception.message, "Burp MCP", JOptionPane.ERROR_MESSAGE)
        }
    }
}
