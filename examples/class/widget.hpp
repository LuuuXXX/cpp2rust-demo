#pragma once
// C++ class header for the cpp2rust-demo class example.
// Extended to demonstrate: constructors, virtual methods, pure virtual,
// static methods, inheritance — matching current cpp2rust-demo capabilities.

// -------------------------------------------------------------------
// Abstract base class (all public methods are pure virtual → #[interface])
// -------------------------------------------------------------------
class Shape {
public:
    virtual ~Shape() {}
    virtual double area() const = 0;
    virtual double perimeter() const = 0;
    virtual const char* name() const = 0;
};

// -------------------------------------------------------------------
// Concrete class with inheritance, non-pure virtual, constructors
// -------------------------------------------------------------------
class Widget : public Shape {
public:
    /// Construct a widget with a given ID.
    Widget(int id);
    ~Widget();

    // Shape interface implementation (non-pure virtual overrides).
    double area() const override;
    double perimeter() const override;
    const char* name() const override;

    /// Update the widget's position (non-virtual).
    void update(double x, double y);

    /// Get the widget's ID (const method).
    int getId() const;

    /// Check whether the widget is visible (const method).
    bool isVisible() const;

    /// Toggle visibility (mutating non-const method).
    void setVisible(bool v);

    /// Global count of Widget instances (static method).
    static int instanceCount();

private:
    int id_;
    double x_ = 0.0;
    double y_ = 0.0;
    bool visible_ = true;
};
