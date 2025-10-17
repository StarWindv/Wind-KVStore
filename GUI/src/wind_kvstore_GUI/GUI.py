import os
import sys

from PyQt5.QtCore import QThread, Qt, pyqtSignal
from PyQt5.QtGui import QFont
from PyQt5.QtWidgets import (
    QAbstractItemView, QApplication,
    QComboBox,
    QDialog, QDialogButtonBox,
    QFileDialog, QGroupBox,
    QHBoxLayout, QHeaderView,
    QLabel, QLineEdit,
    QMainWindow, QMessageBox,
    QProgressBar, QPushButton,
    QSplitter, QTableWidget,
    QTableWidgetItem, QTextEdit,
    QVBoxLayout, QWidget
)

from wind_kvstore.WindKVCore import WindKVCore
from stv_utils import print


Prefix = {
    "info": "\n;2aaa8a;[INFO]",
    "err" : "\n;ee482b;[Err ]",
    "warn": "\n;f4bb44;[Warn]",
}

info = Prefix["info"]
err  = Prefix["err"]
warn = Prefix["warn"]


CRUD = {
    "del": "\n;FA8072;[DEL ]",
    "get": "\n;00FFFF;[GET ]",
    "put": "\n;FFAA33;[PUT ]",
    "update": "\n;AFE1AF;[MOD ]",
}

delete  = CRUD['del']
gets    = CRUD['get']
puts    = CRUD['put']
updates = CRUD['update']


# noinspection PyUnresolvedReferences
class KVConnectionDialog(QDialog):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowTitle("Connect Wind-KVStore")
        self.setModal(True)
        self.resize(500, 200)

        layout = QVBoxLayout(self)

        file_layout = QHBoxLayout()
        file_layout.addWidget(QLabel("KV Path:"))
        self.file_edit = QLineEdit()
        self.file_edit.setPlaceholderText("Select Wind-KVStore File...")
        file_layout.addWidget(self.file_edit)

        self.browse_btn = QPushButton("Browse...")
        self.browse_btn.clicked.connect(self.browse_database)
        file_layout.addWidget(self.browse_btn)

        layout.addLayout(file_layout)

        buttons = QDialogButtonBox(QDialogButtonBox.Ok | QDialogButtonBox.Cancel)
        buttons.accepted.connect(self.accept)
        buttons.rejected.connect(self.reject)
        layout.addWidget(buttons)

    def browse_database(self):
        print(info, "Opening file dialog")
        file_path, _ = QFileDialog.getOpenFileName(
            self,
            "Select Wind-KVStore File",
            os.path.join(os.getcwd(), ".."),
            "Wind (*.wind, *.kv);;All (*.*)",
            "*.wind"
        )
        if file_path:
            self.file_edit.setText(file_path)
            print(info, f"Selected: {file_path}")
        else:
            print(info, "No database file selected")

    def get_database_path(self):
        return self.file_edit.text()

    def get_db_identifier(self):
        return self.identifier_edit.text() or None


# noinspection PyUnresolvedReferences
class AddEditDialog(QDialog):
    def __init__(self, parent=None, mode="add", key_data=None, value_data=None):
        super().__init__(parent)
        self.mode = mode
        self.key_data = key_data
        self.value_data = value_data

        self.setWindowTitle(f"{'Add' if mode == 'add' else 'Edit'} Key-Value")
        self.setModal(True)
        self.resize(500, 300)

        layout = QVBoxLayout(self)

        key_layout = QVBoxLayout()
        key_layout.addWidget(QLabel("Key:"))
        self.key_edit = QTextEdit()
        self.key_edit.setMaximumHeight(80)
        key_layout.addWidget(self.key_edit)
        layout.addLayout(key_layout)

        value_layout = QVBoxLayout()
        value_layout.addWidget(QLabel("Value:"))
        self.value_edit = QTextEdit()
        value_layout.addWidget(self.value_edit)
        layout.addLayout(value_layout)

        encoding_layout = QHBoxLayout()
        encoding_layout.addWidget(QLabel("Encoding:"))
        self.encoding_combo = QComboBox()
        self.encoding_combo.addItems(["UTF-8", "Latin-1", "Base64"])
        encoding_layout.addWidget(self.encoding_combo)
        encoding_layout.addStretch()
        layout.addLayout(encoding_layout)

        if mode == "edit" and key_data:
            print(info, f"Populating edit dialog | key: \"{key_data}\"")
            self.key_edit.setText(key_data)
            self.value_edit.setText(value_data)

        buttons = QDialogButtonBox(QDialogButtonBox.Ok | QDialogButtonBox.Cancel)
        buttons.accepted.connect(self.accept)
        buttons.rejected.connect(self.reject)
        layout.addWidget(buttons)

    def get_data(self):
        key_bytes  = None
        value_bytes = None

        encoding = self.encoding_combo.currentText()
        key_text = self.key_edit.toPlainText()
        value_text = self.value_edit.toPlainText()
        print(info, f"Processing Data | Encoding: {encoding}")

        if encoding == "UTF-8":
            key_bytes = key_text.encode('utf-8')
            value_bytes = value_text.encode('utf-8')
        elif encoding == "Latin-1":
            key_bytes = key_text.encode('latin-1')
            value_bytes = value_text.encode('latin-1')
        elif encoding == "Base64":
            import base64
            try:
                key_bytes = base64.b64decode(key_text)
                value_bytes = base64.b64decode(value_text)
            except Exception as e:
                print(f"Base64 decoding error: {str(e)}")
                QMessageBox.warning(self, "Error", "Base64 Decoding Error")
                return key_bytes, value_bytes

        return key_bytes, value_bytes


# noinspection PyUnresolvedReferences
class DataLoaderThread(QThread):
    data_loaded = pyqtSignal(list)
    progress_updated = pyqtSignal(int)

    def __init__(self, kv_core):
        super().__init__()
        self.kv_core = kv_core
        print(info, "Data loader thread initialized")

    def run(self):
        try:
            print(info, "Loading Data")
            all_data = self.kv_core.get_all()
            print(info, f"Loaded: {len(all_data)}")
            self.data_loaded.emit(all_data)
        except Exception as e:
            error_msg = f"Error loading data: {str(e)}"
            print(err, error_msg)
            self.data_loaded.emit([{"error": error_msg}])


class CustomTableWidget(QTableWidget):
    # noinspection PyTypeChecker
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setAlternatingRowColors(True)
        self.setSelectionBehavior(QAbstractItemView.SelectRows)
        self.setSelectionMode(QAbstractItemView.SingleSelection)
        self.setSortingEnabled(True)
        print(info, "Custom table widget initialized")

        font = QFont()
        font.setPointSize(9)
        self.setFont(font)

        self.setColumnCount(2)
        self.setHorizontalHeaderLabels(["Key", "Value"])

        header = self.horizontalHeader()
        header.setSectionResizeMode(0, QHeaderView.Interactive)
        header.setSectionResizeMode(1, QHeaderView.Stretch)

    def mousePressEvent(self, event):
        index = self.indexAt(event.pos())
        if not index.isValid():
            self.clearSelection()
            self.setCurrentItem(None)
        super().mousePressEvent(event)

    def resizeEvent(self, event):
        super().resizeEvent(event)
        total_width = self.viewport().width()
        self.setColumnWidth(0, int(total_width * 0.3))
        print(info, f"Table resized to:"
                    f"{self.viewport().width()}x{self.viewport().height()}")

    def add_data(self, data):
        self.setRowCount(0)
        print(info, "Clearing existing table data")

        if not data:
            print(warn, "No data to display in table")
            return

        if len(data) == 1 and "error" in data[0]:
            self.setRowCount(1)
            self.setItem(0, 0, QTableWidgetItem("Error"))
            self.setItem(0, 1, QTableWidgetItem(data[0]["error"]))
            print(err, f"Displaying error in table: {data[0]['error']}")
            return

        self.setRowCount(len(data))
        print(info, f"Populating table with {len(data)} entries")

        for row, item in enumerate(data):
            key = item.get("key", "")
            value = item.get("value", "")

            key_item = QTableWidgetItem(key)
            value_item = QTableWidgetItem(value)

            if len(value) > 100:
                value_item.setText(value[:100] + "...")
                value_item.setToolTip(value)
                print(info, f"Truncated long value for key: {key} (length: {len(value)})")

            self.setItem(row, 0, key_item)
            self.setItem(row, 1, value_item)


# noinspection PyUnresolvedReferences
class MainWindow(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("Wind-KVStore GUI")
        self.resize(1200, 800)
        print(info, "Main window initialized")

        self.kv_core = None
        self.current_data = []
        self._initialize()

    def _initialize(self):
        self._setup_ui()
        self._set_connected_state(False)
        print(info, "Main window initialization complete")

    def _setup_ui(self):
        central_widget = QWidget()
        self.setCentralWidget(central_widget)

        layout = QVBoxLayout(central_widget)

        toolbar_layout = QHBoxLayout()

        self.connect_btn = QPushButton("Connect Database")
        self.connect_btn.clicked.connect(self._connect_database)
        toolbar_layout.addWidget(self.connect_btn)

        self.disconnect_btn = QPushButton("Disconnect")
        self.disconnect_btn.clicked.connect(self._disconnect_database)
        toolbar_layout.addWidget(self.disconnect_btn)

        self.refresh_btn = QPushButton("Refresh Data")
        self.refresh_btn.clicked.connect(self._refresh_data)
        toolbar_layout.addWidget(self.refresh_btn)

        toolbar_layout.addStretch()

        self.db_info_label = QLabel("Not connected to database")
        toolbar_layout.addWidget(self.db_info_label)

        layout.addLayout(toolbar_layout)

        splitter = QSplitter(Qt.Horizontal)
        layout.addWidget(splitter)

        left_panel = QWidget()
        left_layout = QVBoxLayout(left_panel)

        single_key_group = QGroupBox("Single Key Operations")
        single_key_layout = QVBoxLayout(single_key_group)

        key_input_layout = QHBoxLayout()
        key_input_layout.addWidget(QLabel("Key:"))
        self.key_edit = QLineEdit()
        self.key_edit.setPlaceholderText("Enter key...")
        key_input_layout.addWidget(self.key_edit)
        single_key_layout.addLayout(key_input_layout)

        value_layout = QVBoxLayout()
        value_layout.addWidget(QLabel("Value:"))
        self.value_display = QTextEdit()
        self.value_display.setReadOnly(True)
        self.value_display.setMaximumHeight(100)
        value_layout.addWidget(self.value_display)
        single_key_layout.addLayout(value_layout)

        encoding_layout = QHBoxLayout()
        encoding_layout.addWidget(QLabel("Display Encoding:"))
        self.display_encoding_combo = QComboBox()
        self.display_encoding_combo.addItems(["UTF-8", "Latin-1", "Base64", "Hex"])
        encoding_layout.addWidget(self.display_encoding_combo)
        encoding_layout.addStretch()
        single_key_layout.addLayout(encoding_layout)

        single_key_buttons_layout = QHBoxLayout()
        self.get_btn = QPushButton("Get")
        self.get_btn.clicked.connect(self._get_value)
        single_key_buttons_layout.addWidget(self.get_btn)

        self.delete_btn = QPushButton("Delete")
        self.delete_btn.clicked.connect(self._delete_key)
        if self.kv_core:
            self._refresh_data()
        single_key_buttons_layout.addWidget(self.delete_btn)

        single_key_layout.addLayout(single_key_buttons_layout)
        left_layout.addWidget(single_key_group)

        batch_operations_group = QGroupBox("Batch Operations")
        batch_operations_layout = QVBoxLayout(batch_operations_group)

        self.add_btn = QPushButton("Add Key-Value Pair")
        self.add_btn.clicked.connect(self._add_key_value)
        batch_operations_layout.addWidget(self.add_btn)

        self.edit_btn = QPushButton("Edit Selected Key-Value")
        self.edit_btn.clicked.connect(self._edit_key_value)
        batch_operations_layout.addWidget(self.edit_btn)

        self.compact_btn = QPushButton("Compact Database")
        self.compact_btn.clicked.connect(self._compact_database)
        batch_operations_layout.addWidget(self.compact_btn)

        batch_operations_layout.addStretch()
        left_layout.addWidget(batch_operations_group)

        db_management_group = QGroupBox("Database Management")
        db_management_layout = QVBoxLayout(db_management_group)

        identifier_layout = QHBoxLayout()
        identifier_layout.addWidget(QLabel("Identifier:"))
        self.identifier_edit = QLineEdit()
        self.identifier_edit.setPlaceholderText("Database identifier...")
        identifier_layout.addWidget(self.identifier_edit)

        self.set_identifier_btn = QPushButton("Set")
        self.set_identifier_btn.clicked.connect(self._set_identifier)
        identifier_layout.addWidget(self.set_identifier_btn)

        db_management_layout.addLayout(identifier_layout)
        left_layout.addWidget(db_management_group)

        right_panel = QWidget()
        right_layout = QVBoxLayout(right_panel)

        search_layout = QHBoxLayout()
        search_layout.addWidget(QLabel("Search:"))
        self.search_edit = QLineEdit()
        self.search_edit.setPlaceholderText("Search in keys and values...")
        self.search_edit.textChanged.connect(self._filter_table)
        search_layout.addWidget(self.search_edit)

        self.progress_bar = QProgressBar()
        self.progress_bar.setVisible(False)
        search_layout.addWidget(self.progress_bar)

        right_layout.addLayout(search_layout)

        self.table_widget = CustomTableWidget()
        right_layout.addWidget(self.table_widget)

        self.status_label = QLabel("Ready")
        right_layout.addWidget(self.status_label)

        splitter.addWidget(left_panel)
        splitter.addWidget(right_panel)
        splitter.setSizes([400, 800])
        print(info, "UI setup completed")

    def _set_connected_state(self, connected):
        self.disconnect_btn.setEnabled(connected)
        self.refresh_btn.setEnabled(connected)
        self.get_btn.setEnabled(connected)
        self.delete_btn.setEnabled(connected)
        self.add_btn.setEnabled(connected)
        self.edit_btn.setEnabled(connected)
        self.compact_btn.setEnabled(connected)
        self.set_identifier_btn.setEnabled(connected)
        self.search_edit.setEnabled(connected)
        self.key_edit.setEnabled(connected)
        self.identifier_edit.setEnabled(connected)

        if not connected:
            self.table_widget.setRowCount(0)
            self.value_display.clear()
            self.db_info_label.setText("Not connected to database")
            self.status_label.setText("Not connected to database")

        print(info, f"Connection state set to: {'connected' if connected else 'disconnected'}")

    def _connect_database(self):
        print(info, "Opening database connection dialog")
        dialog = KVConnectionDialog(self)
        if dialog.exec_() == QDialog.Accepted:
            db_path = dialog.get_database_path()
            self.db_path = db_path
            if not db_path:
                QMessageBox.warning(self, "Warning", "Please select a database file!")
                print("Database connection cancelled - no file selected")
                return

            try:
                print(info, f"Try to connect: {db_path}")
                self.kv_core = WindKVCore(db_path)

                self._set_connected_state(True)
                self.setWindowTitle(f"Wind-KVStore GUI | {os.path.basename(db_path)}")

                identifier = self.kv_core.get_identifier()
                self.db_info_label.setText(f"Database: {os.path.basename(db_path)} | Identifier: {identifier}")
                self.identifier_edit.setText(identifier)

                self._refresh_data()

                self.status_label.setText("Connected successfully")
                print(info, "Connection successful")

            except Exception as e:
                error_msg = f"Failed to connect to database: {str(e)}"
                QMessageBox.critical(self, "Error", error_msg)
                print(err, error_msg)

    def _disconnect_database(self):
        print(info, "Initiating database disconnection")
        if self.kv_core:
            try:
                self.kv_core.close()
                self.kv_core = None
                print(info, "Database connection closed successfully")
            except Exception as e:
                error_msg = f"Error while disconnecting: {str(e)}"
                QMessageBox.warning(self, "Warning", error_msg)
                print(err, error_msg)

        self._set_connected_state(False)
        self.setWindowTitle("Wind-KVStore GUI")
        QMessageBox.information(self, "Success", "Disconnected from database")
        print(info, "Disconnected")

    def _refresh_data(self):
        if not self.kv_core:
            QMessageBox.warning(self, "Warning", "Please connect to a database first!")
            print(warn, "Data refresh cancelled | no database connection")
            return

        self.status_label.setText("Loading data...")
        self.progress_bar.setVisible(True)

        self.loader_thread = DataLoaderThread(self.kv_core)
        self.loader_thread.data_loaded.connect(self._on_data_loaded)
        self.loader_thread.start()
        print(info, "Data loader thread started")

    def _on_data_loaded(self, data):
        self.current_data = data
        self.table_widget.add_data(data)
        self.progress_bar.setVisible(False)

        if len(data) == 1 and "error" in data[0]:
            self.status_label.setText(f"Error loading data: {data[0]['error']}")
        else:
            self.status_label.setText(f"Loaded {len(data)} key-value pairs")

    def _filter_table(self):
        search_text = self.search_edit.text().lower()
        print(f"Filtering table with search text: '{search_text}'")

        if not search_text:
            self.table_widget.add_data(self.current_data)
            self.status_label.setText(f"Showing all {len(self.current_data)} entries")
            return

        filtered_data = [
            item for item in self.current_data
            if search_text in item.get("key", "").lower() or
               search_text in item.get("value", "").lower()
        ]

        self.table_widget.add_data(filtered_data)
        self.status_label.setText(f"Showing {len(filtered_data)} matching entries")
        print(f"Filtering completed. {len(filtered_data)} matches found")

    def _get_value(self):
        key_bytes  = None
        key_text = self.key_edit.text().strip()
        if not key_text:
            QMessageBox.warning(self, "Warning", "Please enter a key!")
            print("Get operation cancelled - no key entered")
            return

        if not self.kv_core:
            QMessageBox.warning(self, "Warning", "Please connect to a database first!")
            print("Get operation cancelled - no database connection")
            return

        try:
            encoding = self.display_encoding_combo.currentText()
            print(gets, f"Key: '{key_text}' | Encoding: {encoding}")

            if encoding == "UTF-8":
                key_bytes = key_text.encode('utf-8')
            elif encoding == "Latin-1":
                key_bytes = key_text.encode('latin-1')
            elif encoding == "Base64":
                import base64
                key_bytes = base64.b64decode(key_text)
            elif encoding == "Hex":
                key_bytes = bytes.fromhex(key_text)
            self.kv_core = WindKVCore(self.db_path)
            value_bytes = self.kv_core.get(key_bytes)

            if value_bytes is None:
                self.value_display.setText("Key does not exist")
                print(warn, f"Key '{key_text}' not found in database")
                return

            if encoding == "UTF-8":
                try:
                    self.value_display.setText(value_bytes.decode('utf-8'))
                except Exception as e:
                    self.value_display.setText("Cannot decode as UTF-8")
                    print(err, str(e))
            elif encoding == "Latin-1":
                self.value_display.setText(value_bytes.decode('latin-1'))
            elif encoding == "Base64":
                import base64
                self.value_display.setText(base64.b64encode(value_bytes).decode('ascii'))
            elif encoding == "Hex":
                self.value_display.setText(value_bytes.hex())

            print(gets, f"Get key: '{key_text}'")

        except Exception as e:
            error_msg = f"Error retrieving value: {str(e)}"
            QMessageBox.warning(self, "Error", error_msg)
            print(err, error_msg)

    def _delete_key(self):
        key_bytes = None
        key_text = self.key_edit.text().strip()
        if not key_text:
            QMessageBox.warning(self, "Warning", "Please enter a key!")
            print(warn, "Delete operation cancelled | No Select")
            return

        if not self.kv_core:
            QMessageBox.warning(self, "Warning", "Please connect to a database first!")
            print("Delete operation cancelled - no database connection")
            return

        reply = QMessageBox.question(
            self, "Confirm Deletion",
            f"Are you sure you want to delete the key '{key_text}'?",
            QMessageBox.Yes | QMessageBox.No
        )

        if reply == QMessageBox.Yes:
            try:
                encoding = self.display_encoding_combo.currentText()
                print(info, f"Try to delete: key: \"{key_text}\" | Encoding: {encoding}")

                if encoding == "UTF-8":
                    key_bytes = key_text.encode('utf-8')
                elif encoding == "Latin-1":
                    key_bytes = key_text.encode('latin-1')
                elif encoding == "Base64":
                    import base64
                    key_bytes = base64.b64decode(key_text)
                elif encoding == "Hex":
                    key_bytes = bytes.fromhex(key_text)

                self.kv_core.delete(key_bytes)
                self.value_display.clear()
                self.status_label.setText(f"Deleted key: {key_text}")
                print(delete, f"Key: '{key_text}'")

                self._refresh_data()

            except Exception as e:
                error_msg = f"Error deleting key: {str(e)}"
                QMessageBox.warning(self, "Error", error_msg)
                print(err, error_msg)
        self.kv_core = WindKVCore(self.db_path)

    def _add_key_value(self):
        if not self.kv_core:
            QMessageBox.warning(self, "Warning", "Please connect to a database first!")
            print("Add operation cancelled - no database connection")
            return

        print(info, "Adding...")
        dialog = AddEditDialog(self, mode="add")
        if dialog.exec_() == QDialog.Accepted:
            key_bytes, value_bytes = dialog.get_data()

            if key_bytes is None or value_bytes is None:
                print(warn, "Add Failed")
                return

            try:
                print(puts, f"{key_bytes}: {value_bytes}")
                self.kv_core.put(key_bytes, value_bytes)
                # print(self.kv_core.get_all())
                self.status_label.setText("Added successfully")
                print(info, "Added")

                self._refresh_data()

            except Exception as e:
                error_msg = f"Error adding key-value pair: {str(e)}"
                QMessageBox.warning(self, "Error", error_msg)
                print(err, error_msg)
        self.kv_core = WindKVCore(self.db_path)

    def _edit_key_value(self):
        current_row = self.table_widget.currentRow()
        if current_row < 0:
            QMessageBox.warning(self, "Warning", "Please select a record first!")
            print("Edit operation cancelled - no record selected")
            return

        if not self.kv_core:
            QMessageBox.warning(self, "Warning", "Please connect to a database first!")
            print("Edit operation cancelled - no database connection")
            return

        key_item = self.table_widget.item(current_row, 0)
        value_item = self.table_widget.item(current_row, 1)

        if not key_item:
            print("Edit operation cancelled - no key found in selected row")
            return

        key_text = key_item.text()
        value_text = value_item.text()
        print(info, f"Preparing to edit | key: \"{key_text}\" | row: \"{current_row}\"")

        if value_text.endswith("..."):
            try:
                key_bytes = key_text.encode('utf-8')
                value_bytes = self.kv_core.get(key_bytes)
                if value_bytes:
                    value_text = value_bytes.decode('utf-8', errors='replace')
                    print("Retrieved full value for editing")
            except Exception as e:
                print(f"Error retrieving full value for editing: {str(e)}")

        dialog = AddEditDialog(self, mode="edit", key_data=key_text, value_data=value_text)
        if dialog.exec_() == QDialog.Accepted:
            key_bytes, value_bytes = dialog.get_data()

            if key_bytes is None or value_bytes is None:
                print("Edit operation cancelled - invalid data")
                return

            try:
                print(updates, f"Updating | {key_bytes}: {value_bytes}")
                self.kv_core.put(key_bytes, value_bytes)
                self.status_label.setText("Key-value pair updated successfully")
                print(info, "Updated")

                self._refresh_data()

            except Exception as e:
                error_msg = f"Error updating key-value pair: {str(e)}"
                QMessageBox.warning(self, "Error", error_msg)
                print(err, error_msg)

    def _compact_database(self):
        if not self.kv_core:
            QMessageBox.warning(self, "Warning", "Please connect to a database first!")
            print("Compaction cancelled - no database connection")
            return

        try:
            print(info, "Compacting Store")
            self.kv_core.compact()
            self.status_label.setText("Compacted")
            QMessageBox.information(self, "Success", "Database compaction completed!")
            print(info, "Compacted")
        except Exception as e:
            error_msg = f"Error compacting database: {str(e)}"
            QMessageBox.warning(self, "Error", error_msg)
            print(err, error_msg)

    def _set_identifier(self):
        if not self.kv_core:
            QMessageBox.warning(self, "Warning", "Please connect to a database first!")
            print("Set identifier cancelled - no database connection")
            return

        identifier = self.identifier_edit.text().strip()
        if not identifier:
            QMessageBox.warning(self, "Warning", "Please enter an identifier!")
            print("Set identifier cancelled - no identifier entered")
            return

        try:
            print(info, f"Set db ID: {identifier}")
            self.kv_core.set_identifier(identifier)
            self.db_info_label.setText(f"Database: {os.path.basename(self.kv_core.path)} | Identifier: {identifier}")
            self.status_label.setText(f"Identifier set to: {identifier}")
            QMessageBox.information(self, "Success", f"Database identifier set to: {identifier}")
            print(info, f"ID set successfully")
        except Exception as e:
            error_msg = f"Error setting identifier: {str(e)}"
            QMessageBox.warning(self, "Error", error_msg)
            print(err, error_msg)


def main():
    print(info, "Run Wind-KVStore GUI App")
    app = QApplication(sys.argv)

    window = MainWindow()
    window.show()

    code = app.exec_()
    print(info, "Quit App")
    sys.exit(code)


if __name__ == "__main__":
    main()
