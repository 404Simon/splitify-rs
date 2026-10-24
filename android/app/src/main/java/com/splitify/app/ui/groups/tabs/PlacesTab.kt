package com.splitify.app.ui.groups.tabs

// TODO(phase 4d): Interactive MapLibre map rendering is deferred for now; this
// tab only lists the API's marker data and hands coordinates off to an external
// maps app.

import android.content.ActivityNotFoundException
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.widget.Toast
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.background
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import com.splitify.app.ui.components.EmptyState
import com.splitify.app.ui.components.Pill
import com.splitify.app.ui.components.SplitifyCard
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material.icons.filled.Place
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExtendedFloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.core.net.toUri
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewModelScope
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.lifecycle.viewmodel.initializer
import androidx.lifecycle.viewmodel.viewModelFactory
import com.splitify.app.data.dto.CreateMarkerRequest
import com.splitify.app.data.dto.MapMarkerDto
import com.splitify.app.data.maps.MapMarkerRepository
import com.splitify.app.ui.components.ErrorBanner
import com.splitify.app.ui.components.LoadingScreen
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class PlacesUiState(
    val markers: List<MapMarkerDto> = emptyList(),
    val isLoading: Boolean = true,
    /** Local write failure (the marker list itself is never network-backed). */
    val error: String? = null,
    val showFormDialog: Boolean = false,
    val editingMarkerId: Long? = null,
    val isSaving: Boolean = false,
    val formError: String? = null,
    val deleteTarget: MapMarkerDto? = null,
)

class PlacesViewModel(
    private val groupId: Long,
    private val mapMarkerRepository: MapMarkerRepository,
) : ViewModel() {

    private val _uiState = MutableStateFlow(PlacesUiState())
    val uiState: StateFlow<PlacesUiState> = _uiState.asStateFlow()

    init {
        viewModelScope.launch {
            mapMarkerRepository.observeMarkers(groupId).collect { markers ->
                _uiState.update { it.copy(markers = markers, isLoading = false) }
            }
        }
    }

    fun dismissError() {
        _uiState.update { it.copy(error = null) }
    }

    fun openCreate() {
        _uiState.update {
            it.copy(showFormDialog = true, editingMarkerId = null, formError = null)
        }
    }

    fun openEdit(marker: MapMarkerDto) {
        _uiState.update {
            it.copy(showFormDialog = true, editingMarkerId = marker.id, formError = null)
        }
    }

    fun dismissForm() {
        if (_uiState.value.isSaving) return
        _uiState.update {
            it.copy(showFormDialog = false, editingMarkerId = null, formError = null)
        }
    }

    fun save(
        name: String,
        description: String,
        address: String,
        emoji: String,
        latitude: String,
        longitude: String,
    ) {
        val trimmedName = name.trim()
        val parsedLatitude = parseLatitude(latitude)
        val parsedLongitude = parseLongitude(longitude)
        val validationError = when {
            trimmedName.isEmpty() -> "Enter a place name."
            parsedLatitude == null -> "Enter a latitude between -90 and 90."
            parsedLongitude == null -> "Enter a longitude between -180 and 180."
            else -> null
        }
        if (validationError != null) {
            _uiState.update { it.copy(formError = validationError) }
            return
        }
        if (_uiState.value.isSaving) return
        val validLatitude = parsedLatitude ?: return
        val validLongitude = parsedLongitude ?: return
        _uiState.update { it.copy(isSaving = true, formError = null) }

        val request = CreateMarkerRequest(
            name = trimmedName,
            description = description.trim().ifEmpty { null },
            address = address.trim().ifEmpty { null },
            emoji = emoji.trim().ifEmpty { "📍" },
            latitude = validLatitude,
            longitude = validLongitude,
        )

        viewModelScope.launch {
            val editingId = _uiState.value.editingMarkerId
            val result: Result<Unit> = if (editingId != null) {
                mapMarkerRepository.updateMarker(groupId, editingId, request)
            } else {
                mapMarkerRepository.createMarker(groupId, request).map { }
            }
            result
                .onSuccess {
                    _uiState.update {
                        it.copy(
                            isSaving = false,
                            showFormDialog = false,
                            editingMarkerId = null,
                            formError = null,
                        )
                    }
                }
                .onFailure {
                    _uiState.update {
                        it.copy(isSaving = false, formError = SAVE_FAILED_MESSAGE)
                    }
                }
        }
    }

    fun requestDelete(marker: MapMarkerDto) {
        _uiState.update { it.copy(deleteTarget = marker) }
    }

    fun dismissDelete() {
        _uiState.update { it.copy(deleteTarget = null) }
    }

    fun confirmDelete() {
        val target = _uiState.value.deleteTarget ?: return
        // The delete is applied to the local snapshot right away.
        _uiState.update { it.copy(deleteTarget = null) }
        viewModelScope.launch {
            mapMarkerRepository.deleteMarker(groupId, target.id)
                .onFailure {
                    _uiState.update { it.copy(error = DELETE_FAILED_MESSAGE) }
                }
        }
    }

    companion object {
        private const val SAVE_FAILED_MESSAGE = "Could not save the place. Please try again."
        private const val DELETE_FAILED_MESSAGE = "Could not delete the place. Please try again."

        fun factory(
            groupId: Long,
            mapMarkerRepository: MapMarkerRepository,
        ): ViewModelProvider.Factory = viewModelFactory {
            initializer { PlacesViewModel(groupId, mapMarkerRepository) }
        }
    }
}

@Composable
fun PlacesTab(
    groupId: Long,
    currentUserId: Long?,
    groupCreatedBy: Long,
    mapMarkerRepository: MapMarkerRepository,
) {
    val viewModel: PlacesViewModel = viewModel(
        key = "places-$groupId",
        factory = PlacesViewModel.factory(groupId, mapMarkerRepository),
    )
    val state by viewModel.uiState.collectAsStateWithLifecycle()
    val context = LocalContext.current

    Box(modifier = Modifier.fillMaxSize()) {
        Column(modifier = Modifier.fillMaxSize()) {
            state.error?.let { message ->
                ErrorBanner(
                    message = message,
                    onDismiss = { viewModel.dismissError() },
                    modifier = Modifier.padding(horizontal = 20.dp, vertical = 8.dp),
                )
            }
            Box(modifier = Modifier.fillMaxWidth().weight(1f)) {
                when {
                    state.isLoading && state.markers.isEmpty() ->
                        LoadingScreen(message = "Loading places…")

                    state.markers.isEmpty() -> Box(
                        Modifier.fillMaxSize(),
                        contentAlignment = Alignment.Center,
                    ) {
                        EmptyState(
                            icon = Icons.Default.Place,
                            title = "No places yet",
                            message = "Pin the cafés, bars and spots your group wants to visit.",
                            accent = MaterialTheme.colorScheme.tertiary,
                            accentContainer = MaterialTheme.colorScheme.tertiaryContainer,
                            actionLabel = "Add a place",
                            onAction = { viewModel.openCreate() },
                        )
                    }

                    else -> LazyColumn(
                        modifier = Modifier.fillMaxSize(),
                        contentPadding = PaddingValues(
                            start = 20.dp,
                            end = 20.dp,
                            top = 4.dp,
                            bottom = 96.dp,
                        ),
                        verticalArrangement = Arrangement.spacedBy(12.dp),
                    ) {
                        items(state.markers, key = { it.id }) { marker ->
                            PlaceCard(
                                marker = marker,
                                canManage = currentUserId != null &&
                                    (marker.createdBy == currentUserId ||
                                        groupCreatedBy == currentUserId),
                                onOpenInMaps = { openInMaps(context, marker) },
                                onEdit = { viewModel.openEdit(marker) },
                                onDelete = { viewModel.requestDelete(marker) },
                            )
                        }
                    }
                }
            }
        }

        if (!state.isLoading || state.markers.isNotEmpty()) {
            ExtendedFloatingActionButton(
                onClick = { viewModel.openCreate() },
                modifier = Modifier
                    .align(Alignment.BottomEnd)
                    .padding(16.dp),
                shape = CircleShape,
                containerColor = MaterialTheme.colorScheme.primary,
                contentColor = MaterialTheme.colorScheme.onPrimary,
                icon = { Icon(Icons.Default.Add, contentDescription = null) },
                text = { Text("New place") },
            )
        }
    }

    if (state.showFormDialog) {
        val editingMarker = state.markers.firstOrNull { it.id == state.editingMarkerId }
        PlaceFormDialog(
            editingMarker = editingMarker,
            isSaving = state.isSaving,
            error = state.formError,
            onDismiss = { viewModel.dismissForm() },
            onSave = { name, description, address, emoji, latitude, longitude ->
                viewModel.save(name, description, address, emoji, latitude, longitude)
            },
        )
    }

    state.deleteTarget?.let { marker ->
        AlertDialog(
            onDismissRequest = { viewModel.dismissDelete() },
            shape = RoundedCornerShape(28.dp),
            title = { Text("Delete place") },
            text = { Text("Delete \"${marker.name}\"? This cannot be undone.") },
            confirmButton = {
                TextButton(onClick = { viewModel.confirmDelete() }) {
                    Text("Delete", color = MaterialTheme.colorScheme.error)
                }
            },
            dismissButton = {
                TextButton(onClick = { viewModel.dismissDelete() }) {
                    Text("Cancel")
                }
            },
        )
    }
}

@Composable
private fun PlaceCard(
    marker: MapMarkerDto,
    canManage: Boolean,
    onOpenInMaps: () -> Unit,
    onEdit: () -> Unit,
    onDelete: () -> Unit,
) {
    var menuExpanded by remember { mutableStateOf(false) }

    SplitifyCard(contentPadding = PaddingValues(16.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Box(
                modifier = Modifier
                    .size(48.dp)
                    .clip(RoundedCornerShape(16.dp))
                    .background(MaterialTheme.colorScheme.tertiaryContainer),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    text = marker.emoji.ifBlank { "📍" },
                    fontSize = 24.sp,
                )
            }
            Spacer(Modifier.width(12.dp))
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = marker.name,
                    style = MaterialTheme.typography.titleMedium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Text(
                    text = "Added by ${marker.creatorUsername}",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            if (canManage) {
                Box {
                    IconButton(onClick = { menuExpanded = true }) {
                        Icon(
                            imageVector = Icons.Default.MoreVert,
                            contentDescription = "Place options",
                            tint = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                    DropdownMenu(
                        expanded = menuExpanded,
                        onDismissRequest = { menuExpanded = false },
                    ) {
                        DropdownMenuItem(
                            text = { Text("Edit") },
                            leadingIcon = { Icon(Icons.Default.Edit, contentDescription = null) },
                            onClick = {
                                menuExpanded = false
                                onEdit()
                            },
                        )
                        DropdownMenuItem(
                            text = { Text("Delete") },
                            leadingIcon = {
                                Icon(
                                    imageVector = Icons.Default.Delete,
                                    contentDescription = null,
                                    tint = MaterialTheme.colorScheme.error,
                                )
                            },
                            onClick = {
                                menuExpanded = false
                                onDelete()
                            },
                        )
                    }
                }
            }
        }

        marker.address?.takeIf { it.isNotBlank() }?.let { address ->
            Spacer(Modifier.height(10.dp))
            Row(verticalAlignment = Alignment.CenterVertically) {
                Icon(
                    imageVector = Icons.Default.Place,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.size(15.dp),
                )
                Spacer(Modifier.width(6.dp))
                Text(
                    text = address,
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
        marker.description?.takeIf { it.isNotBlank() }?.let { description ->
            Spacer(Modifier.height(8.dp))
            Text(
                text = description,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }

        Spacer(Modifier.height(10.dp))
        Row(verticalAlignment = Alignment.CenterVertically) {
            Pill(
                text = coordinateLabel(marker.latitude, marker.longitude),
                containerColor = MaterialTheme.colorScheme.surfaceContainerHigh,
                contentColor = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Spacer(Modifier.weight(1f))
            TextButton(
                onClick = onOpenInMaps,
                contentPadding = PaddingValues(horizontal = 8.dp, vertical = 4.dp),
            ) {
                Text("Open in Maps")
            }
        }
    }
}

@Composable
private fun PlaceFormDialog(
    editingMarker: MapMarkerDto?,
    isSaving: Boolean,
    error: String?,
    onDismiss: () -> Unit,
    onSave: (String, String, String, String, String, String) -> Unit,
) {
    var name by remember(editingMarker?.id) { mutableStateOf(editingMarker?.name.orEmpty()) }
    var description by remember(editingMarker?.id) {
        mutableStateOf(editingMarker?.description.orEmpty())
    }
    var address by remember(editingMarker?.id) { mutableStateOf(editingMarker?.address.orEmpty()) }
    var emoji by remember(editingMarker?.id) { mutableStateOf(editingMarker?.emoji ?: "📍") }
    var latitude by remember(editingMarker?.id) {
        mutableStateOf(editingMarker?.latitude?.toString().orEmpty())
    }
    var longitude by remember(editingMarker?.id) {
        mutableStateOf(editingMarker?.longitude?.toString().orEmpty())
    }

    val latitudeValid = parseLatitude(latitude) != null
    val longitudeValid = parseLongitude(longitude) != null

    AlertDialog(
        onDismissRequest = onDismiss,
        shape = RoundedCornerShape(28.dp),
        title = { Text(if (editingMarker == null) "New place" else "Edit place") },
        text = {
            Column(
                modifier = Modifier
                    .heightIn(max = 460.dp)
                    .verticalScroll(rememberScrollState()),
                verticalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Box(
                        modifier = Modifier
                            .size(56.dp)
                            .clip(RoundedCornerShape(18.dp))
                            .background(MaterialTheme.colorScheme.tertiaryContainer),
                        contentAlignment = Alignment.Center,
                    ) {
                        Text(text = emoji.ifBlank { "📍" }, fontSize = 26.sp)
                    }
                    Spacer(Modifier.width(12.dp))
                    OutlinedTextField(
                        value = emoji,
                        onValueChange = { emoji = it },
                        label = { Text("Emoji") },
                        singleLine = true,
                        enabled = !isSaving,
                        shape = RoundedCornerShape(14.dp),
                        modifier = Modifier.weight(1f),
                    )
                }
                OutlinedTextField(
                    value = name,
                    onValueChange = { name = it },
                    label = { Text("Name") },
                    singleLine = true,
                    enabled = !isSaving,
                    shape = RoundedCornerShape(14.dp),
                    modifier = Modifier.fillMaxWidth(),
                )
                OutlinedTextField(
                    value = address,
                    onValueChange = { address = it },
                    label = { Text("Address (optional)") },
                    singleLine = true,
                    enabled = !isSaving,
                    shape = RoundedCornerShape(14.dp),
                    modifier = Modifier.fillMaxWidth(),
                )
                OutlinedTextField(
                    value = description,
                    onValueChange = { description = it },
                    label = { Text("Note (optional)") },
                    enabled = !isSaving,
                    minLines = 2,
                    shape = RoundedCornerShape(14.dp),
                    modifier = Modifier.fillMaxWidth(),
                )
                Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                    OutlinedTextField(
                        value = latitude,
                        onValueChange = { latitude = it },
                        label = { Text("Latitude") },
                        singleLine = true,
                        enabled = !isSaving,
                        isError = latitude.isNotBlank() && !latitudeValid,
                        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
                        shape = RoundedCornerShape(14.dp),
                        modifier = Modifier.weight(1f),
                    )
                    OutlinedTextField(
                        value = longitude,
                        onValueChange = { longitude = it },
                        label = { Text("Longitude") },
                        singleLine = true,
                        enabled = !isSaving,
                        isError = longitude.isNotBlank() && !longitudeValid,
                        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
                        shape = RoundedCornerShape(14.dp),
                        modifier = Modifier.weight(1f),
                    )
                }
                error?.let {
                    Text(
                        text = it,
                        color = MaterialTheme.colorScheme.error,
                        style = MaterialTheme.typography.bodySmall,
                    )
                }
            }
        },
        confirmButton = {
            TextButton(
                onClick = {
                    onSave(name, description, address, emoji, latitude, longitude)
                },
                enabled = !isSaving &&
                    name.isNotBlank() &&
                    latitudeValid &&
                    longitudeValid,
            ) {
                if (isSaving) {
                    CircularProgressIndicator(
                        modifier = Modifier.size(18.dp),
                        strokeWidth = 2.dp,
                    )
                } else {
                    Text(if (editingMarker == null) "Create" else "Save")
                }
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss, enabled = !isSaving) {
                Text("Cancel")
            }
        },
    )
}

private fun coordinateLabel(latitude: Double, longitude: Double): String =
    String.format(java.util.Locale.getDefault(), "%.4f, %.4f", latitude, longitude)

private fun parseLatitude(value: String): Double? =
    value.trim().toDoubleOrNull()?.takeIf { it.isFinite() && it in -90.0..90.0 }

private fun parseLongitude(value: String): Double? =
    value.trim().toDoubleOrNull()?.takeIf { it.isFinite() && it in -180.0..180.0 }

private fun openInMaps(context: Context, marker: MapMarkerDto) {
    val coordinates = "${marker.latitude},${marker.longitude}"
    val label = Uri.encode(marker.name)
    val geoUri = "geo:$coordinates?q=$coordinates($label)".toUri()
    try {
        context.startActivity(Intent(Intent.ACTION_VIEW, geoUri))
    } catch (_: ActivityNotFoundException) {
        val fallback = "https://www.google.com/maps/search/?api=1&query=$coordinates".toUri()
        try {
            context.startActivity(Intent(Intent.ACTION_VIEW, fallback))
        } catch (_: ActivityNotFoundException) {
            Toast.makeText(context, "No maps app found", Toast.LENGTH_SHORT).show()
        }
    }
}
