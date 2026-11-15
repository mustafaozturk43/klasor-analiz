import { Component, type ErrorInfo, type ReactNode } from "react";

interface Props { children: ReactNode }
interface State { error: Error | null }

/**
 * Bir ekran cizilirken hata firlatirsa bos pencere yerine aciklama ve yeniden
 * deneme dugmesi gosterir. Dosya islemleri Rust tarafinda yurur; arayuz hatasi
 * devam eden bir tasimayi yarida kesmez ve Islem Gecmisi korunur.
 */
export default class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("Arayüz hatası:", error, info.componentStack);
  }

  render() {
    const { error } = this.state;
    if (!error) return this.props.children;
    return <div className="app-shell"><main className="shell-width workspace" role="alert">
      <section className="panel p-4">
        <h1>Ekran çizilirken bir hata oluştu</h1>
        <p className="hint">Dosyalarınıza dokunulmadı; yapılan işlemler İşlem Geçmişi’nde duruyor. Yeniden deneyebilir veya uygulamayı yeniden başlatabilirsiniz.</p>
        <pre className="text-xs break-all whitespace-pre-wrap">{error.message}</pre>
        <div className="inline-actions">
          <button type="button" className="btn" onClick={() => this.setState({ error: null })}>Yeniden dene</button>
          <button type="button" className="btn" onClick={() => window.location.reload()}>Arayüzü yeniden yükle</button>
        </div>
      </section>
    </main></div>;
  }
}
