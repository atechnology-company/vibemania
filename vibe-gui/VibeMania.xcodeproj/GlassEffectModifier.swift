import SwiftUI

// MARK: - Glass Effect Container

struct GlassEffectContainer<Content: View>: View {
    let spacing: CGFloat
    @ViewBuilder let content: () -> Content
    
    var body: some View {
        content()
            .padding()
            .background {
                RoundedRectangle(cornerRadius: 16, style: .continuous)
                    .fill(.ultraThinMaterial)
            }
            .clipShape(RoundedRectangle(cornerRadius: 16, style: .continuous))
    }
}

// MARK: - Glass Effect Style

struct GlassEffectStyle {
    enum Prominence {
        case regular
        case thin
        case ultraThin
        
        var material: Material {
            switch self {
            case .regular: return .regular
            case .thin: return .thin
            case .ultraThin: return .ultraThinMaterial
            }
        }
    }
    
    let prominence: Prominence
    let tintColor: Color?
    let isInteractive: Bool
    
    static var regular: GlassEffectStyle {
        GlassEffectStyle(prominence: .regular, tintColor: nil, isInteractive: false)
    }
    
    func tint(_ color: Color) -> GlassEffectStyle {
        GlassEffectStyle(prominence: prominence, tintColor: color, isInteractive: isInteractive)
    }
    
    func interactive() -> GlassEffectStyle {
        GlassEffectStyle(prominence: prominence, tintColor: tintColor, isInteractive: true)
    }
}

// MARK: - Glass Effect Modifier

struct GlassEffectModifier: ViewModifier {
    let style: GlassEffectStyle
    let shape: AnyShape
    
    @State private var isHovering = false
    
    func body(content: Content) -> some View {
        content
            .background {
                ZStack {
                    // Base glass material
                    shape
                        .fill(style.prominence.material)
                    
                    // Color tint overlay
                    if let tintColor = style.tintColor {
                        shape
                            .fill(tintColor.opacity(isHovering && style.isInteractive ? 0.2 : 0.1))
                    }
                }
            }
            .clipShape(shape)
            .overlay {
                // Subtle border for definition
                shape
                    .strokeBorder(
                        .white.opacity(isHovering && style.isInteractive ? 0.3 : 0.15),
                        lineWidth: isHovering && style.isInteractive ? 1.5 : 1
                    )
            }
            .shadow(
                color: .black.opacity(isHovering && style.isInteractive ? 0.15 : 0.08),
                radius: isHovering && style.isInteractive ? 12 : 8,
                y: isHovering && style.isInteractive ? 6 : 4
            )
            .scaleEffect(isHovering && style.isInteractive ? 1.02 : 1.0)
            .animation(.spring(response: 0.3, dampingFraction: 0.7), value: isHovering)
            .onHover { hovering in
                if style.isInteractive {
                    isHovering = hovering
                }
            }
    }
}

// MARK: - Shape Type Erasure

struct AnyShape: Shape {
    private let _path: (CGRect) -> Path
    
    init<S: Shape>(_ shape: S) {
        _path = { rect in
            shape.path(in: rect)
        }
    }
    
    func path(in rect: CGRect) -> Path {
        _path(rect)
    }
}

// MARK: - View Extension

extension View {
    func glassEffect<S: Shape>(
        _ style: GlassEffectStyle,
        in shape: S
    ) -> some View {
        modifier(GlassEffectModifier(style: style, shape: AnyShape(shape)))
    }
}

// MARK: - Convenience Shape Extensions

extension Shape where Self == RoundedRectangle {
    static func rect(cornerRadius: CGFloat) -> RoundedRectangle {
        RoundedRectangle(cornerRadius: cornerRadius, style: .continuous)
    }
}
